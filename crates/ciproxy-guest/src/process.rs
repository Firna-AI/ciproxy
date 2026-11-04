//! Linux runner processes with drained output and observable readiness.

use std::path::PathBuf;
use std::process::Stdio;

use async_trait::async_trait;
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::secret::Secret;
use rustix::io::Errno;
use rustix::process::{Pid, Signal, kill_process_group};
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::process::{Child, ChildStdout, Command};

use crate::boundary::{EventFuture, Power, Runner, RunnerEvent};
use crate::error::{Error, ErrorOptionExt, ErrorResultExt, Result};

pub(crate) struct LinuxRunner {
    pub(crate) entry: PathBuf,
    process: Option<Running>,
}

struct Running {
    child: Child,
    output: Lines<BufReader<ChildStdout>>,
    output_open: bool,
    ready: bool,
}

impl LinuxRunner {
    pub(crate) fn new(entry: PathBuf) -> Self {
        Self {
            entry,
            process: None,
        }
    }
}

#[async_trait]
impl Runner for LinuxRunner {
    async fn start(&mut self, registration: Secret) -> Result<()> {
        let mut child = Command::new("/usr/sbin/runuser")
            .args(["--user", "runner", "--"])
            .arg(&self.entry)
            .arg(registration.expose_secret())
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
        });
        Ok(())
    }

    fn next_event(&mut self) -> EventFuture<'_> {
        Box::pin(async move {
            let running = self.process.as_mut().ok_or_internal("runner not started")?;
            loop {
                tokio::select! {
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
