//! Linux runner processes with drained output and observable readiness.

use std::path::PathBuf;
use std::process::Stdio;

use async_trait::async_trait;
use ciproxy_protocol::guest::{JobDecision, JobPreparation};
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::secret::Secret;
use rustix::io::Errno;
use rustix::process::{Pid, Signal, kill_process_group};
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::process::{Child, ChildStdout, Command};

use crate::assignment::{Assignments, Inbox, SocketAssignments};
use crate::guest_setup::{GuestSetup, PreloadedDocker};
use crate::runner_assignment::{AssignedJob, WaitingJob};

use crate::boundary::{EventFuture, Power, Runner, RunnerEvent};
use crate::error::{Error, ErrorOptionExt, ErrorResultExt, Result};

pub(crate) struct LinuxRunner {
    pub(crate) setup: Box<dyn GuestSetup>,
    pub(crate) entry: PathBuf,
    process: Option<Running>,
    pub(crate) handoff: bool,
}

struct Running {
    child: Child,
    output: Lines<BufReader<ChildStdout>>,
    output_open: bool,
    ready: bool,
    assignments: Option<Box<dyn Assignments>>,
    assignment: Option<Box<dyn AssignedJob>>,
}

impl LinuxRunner {
    pub(crate) fn new(entry: PathBuf, handoff: bool, setup: Box<dyn GuestSetup>) -> Self {
        Self {
            setup,
            entry,
            process: None,
            handoff,
        }
    }
}

#[async_trait]
impl Runner for LinuxRunner {
    async fn start(&mut self, registration: Secret) -> Result<()> {
        let preloaded = if self.handoff {
            self.setup.inspect().await
        } else {
            PreloadedDocker::default()
        };
        let listener = if self.handoff {
            Some(crate::handoff::listen().await?)
        } else {
            None
        };
        let mut command = Command::new("/usr/sbin/runuser");
        command
            .args(["--user", "runner", "--preserve-environment", "--"])
            .arg(&self.entry)
            .env(
                "ACTIONS_RUNNER_INPUT_JITCONFIG",
                registration.expose_secret(),
            )
            .env(
                "CIPROXY_ASSIGNMENT_SOCKET",
                if self.handoff {
                    "/run/ciproxy/assignment.sock"
                } else {
                    ""
                },
            );
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0)
            .kill_on_drop(true)
            .spawn()
            .context("start stock runner")?;
        let output = child.stdout.take().ok_or_internal("runner stdout")?;
        self.process = Some(Running {
            child,
            output: BufReader::new(output).lines(),
            output_open: true,
            ready: false,
            assignments: listener.map(|listener| {
                Box::new(Inbox::new(Box::new(SocketAssignments {
                    listener,
                    preloaded,
                }))) as Box<dyn Assignments>
            }),
            assignment: None,
        });
        Ok(())
    }

    fn next_event(&mut self) -> EventFuture<'_> {
        Box::pin(async move {
            let running = self.process.as_mut().ok_or_internal("runner not started")?;
            loop {
                tokio::select! {
                    incoming = async { match &mut running.assignments { Some(assignments) => assignments.receive().await, None => std::future::pending().await } }, if running.assignments.is_some() => {
                        match incoming {
                            Ok(assignment) => {
                                match &mut running.assignment {
                                    Some(current) => {
                                        if !current.reconnect(assignment) { continue; }
                                    }
                                    None => running.assignment = Some(Box::new(WaitingJob::new(assignment))),
                                }
                                if let Some(current) = &running.assignment {
                                    let handoff = current.handoff();
                                    return Ok(RunnerEvent::Starting { job_id: handoff.job_id.clone(), runtime: handoff.runtime.clone() });
                                }
                            }
                            Err(error) => { tracing::warn!(%error,"trusted cache handoff unavailable; runner keeps GitHub fallback"); }
                        }
                    },
                    status = running.child.wait() => {
                        return Ok(RunnerEvent::Exited(RunnerExit {
                            code: status.context("wait for runner")?.code(),
                        }));
                    },
                    line = running.output.next_line(), if running.output_open => {
                        match line.context("read runner readiness")? {
                            Some(line) if !running.ready && line.ends_with(": Listening for Jobs") => {
                                running.ready = true;
                                return Ok(RunnerEvent::Ready);
                            }
                            Some(_) => {}
                            None => running.output_open = false,
                        }
                    },
                }
            }
        })
    }

    async fn stop(&mut self) -> Result<RunnerExit> {
        let running = self.process.as_mut().ok_or_internal("runner not started")?;
        if let Some(id) = running.child.id() {
            let pid = Pid::from_raw(id.cast_signed()).ok_or_internal("runner pid")?;
            match kill_process_group(pid, Signal::TERM) {
                Ok(()) | Err(Errno::SRCH) => {}
                Err(error) => Err(error).context("stop runner group")?,
            }
        }
        let status = running.child.wait().await.context("reap stopped runner")?;
        Ok(RunnerExit {
            code: status.code(),
        })
    }

    async fn prepare(&mut self, job_id: JobId, decision: JobDecision) -> Result<JobPreparation> {
        let running = self.process.as_mut().ok_or_internal("runner not started")?;
        let assigned = running
            .assignment
            .as_mut()
            .ok_or_internal("job assignment unavailable")?;
        if assigned.handoff().job_id != job_id {
            return Err(Error::PoweroffFailed);
        }
        assigned.decide(decision).await
    }
}

pub(crate) struct LinuxPower;

#[async_trait]
impl Power for LinuxPower {
    async fn shutdown(&mut self) -> Result<()> {
        let status = Command::new("/sbin/poweroff")
            .status()
            .await
            .context("power off job VM")?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::PoweroffFailed)
        }
    }
}

pub(crate) struct NamespacePower;

#[async_trait]
impl Power for NamespacePower {
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
