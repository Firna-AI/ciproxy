//! Temporary reservations used only within one pure decision call.

use std::collections::HashSet;

use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::resources::VmSize;

use crate::actions::{Action, Destination, StartReason, StartRunner, StopReason};
use crate::boat::{Sandbox, SandboxId};
use crate::capacity::Usage;
use crate::snapshot::{Label, Runner, RunnerState, Snapshot, WaitingJob};

pub(crate) struct Plan {
    pub(crate) actions: Vec<Action>,
    pub(crate) retiring: HashSet<RunnerId>,
    pub(crate) claimed: HashSet<RunnerId>,
    pub(crate) deleted: Vec<SandboxId>,
    pub(crate) usage: Vec<Usage>,
    pub(crate) reserved: Vec<Usage>,
}

impl Plan {
    pub(crate) fn new(snapshot: &Snapshot) -> Self {
        Self {
            actions: Vec::new(),
            retiring: HashSet::new(),
            claimed: HashSet::new(),
            deleted: Vec::new(),
            usage: snapshot.machines.iter().map(Usage::for_machine).collect(),
            reserved: vec![Usage::default(); snapshot.machines.len()],
        }
    }

    pub(crate) fn stop(&mut self, runner: &Runner, reason: StopReason) {
        if self.retiring.insert(runner.id.clone()) {
            self.actions.push(Action::StopRunner {
                runner_id: runner.id.clone(),
                reason,
            });
        }
    }

    pub(crate) fn delete(&mut self, sandbox: &Sandbox, reason: StopReason) {
        if !self.deleted.contains(&sandbox.id) {
            self.deleted.push(sandbox.id.clone());
            if let Some(retained) = &sandbox.runner {
                self.retiring.insert(retained.runner.id.clone());
            }
            self.actions.push(Action::DeleteSandbox {
                sandbox_id: sandbox.id.clone(),
                reason,
            });
        }
    }

    pub(crate) fn start(
        &mut self,
        job: &WaitingJob,
        label: &Label,
        size: VmSize,
        destination: Destination,
        reason: StartReason,
    ) {
        self.actions.push(Action::StartRunner(StartRunner {
            organization_id: job.organization_id.clone(),
            label: label.name.clone(),
            requested_size: label.size,
            size,
            destination,
            reason,
        }));
    }
}

pub(crate) fn idle(runner: &Runner) -> bool {
    matches!(runner.state, RunnerState::Starting | RunnerState::Ready)
}

pub(crate) fn warm_candidate(runner: &Runner) -> bool {
    runner.label.0 == "ciproxy" && idle(runner)
}

pub(crate) fn label_for<'a>(snapshot: &'a Snapshot, runner: &Runner) -> Option<&'a Label> {
    snapshot
        .labels
        .iter()
        .find(|label| label.name == runner.label)
}
