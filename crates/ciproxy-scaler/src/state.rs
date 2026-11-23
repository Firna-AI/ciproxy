//! Temporary reservations used only within one pure decision call.

use std::collections::HashSet;

use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::resources::VmSize;

use crate::actions::{Action, Destination, StartReason, StartRunner, StopReason};
use crate::capacity::Usage;
use crate::provider::{MonthlySpend, ProviderId, Sandbox, SandboxId, StartBound};
use crate::snapshot::{Label, Runner, RunnerState, Snapshot, WaitingJob, warm_label};

pub(crate) struct Plan {
    pub(crate) actions: Vec<Action>,
    pub(crate) retiring: HashSet<RunnerId>,
    pub(crate) claimed: HashSet<RunnerId>,
    deleted: Vec<(ProviderId, SandboxId)>,
    pub(crate) usage: Vec<Usage>,
    pub(crate) reserved: Vec<Usage>,
    provider_spend: Vec<(ProviderId, MonthlySpend)>,
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
            provider_spend: snapshot
                .providers
                .iter()
                .map(|provider| (provider.id, provider.facts.spend))
                .collect(),
        }
    }

    pub(crate) fn can_reserve(&self, provider: ProviderId, bound: StartBound) -> bool {
        self.provider_spend
            .iter()
            .any(|(id, spend)| *id == provider && spend.can_cover(bound) == Some(true))
    }

    pub(crate) fn reserve_provider(&mut self, provider: ProviderId, bound: StartBound) -> bool {
        self.provider_spend
            .iter_mut()
            .find(|(id, _)| *id == provider)
            .is_some_and(|(_, spend)| spend.reserve(bound))
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
        let key = (sandbox.provider, sandbox.id.clone());
        if !self.deleted.contains(&key) {
            self.deleted.push(key);
            if let Some(retained) = &sandbox.runner {
                self.retiring.insert(retained.runner.id.clone());
            }
            self.actions.push(Action::DeleteSandbox {
                provider: sandbox.provider,
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
        self.actions.push(Action::StartRunner(Box::new(StartRunner {
            target: job.target.clone(),
            organization_id: job.target.organization_id.clone(),
            label: label.name.clone(),
            platform: label.platform,
            requested_size: label.size,
            size,
            destination,
            reason,
        })));
    }
}

pub(crate) fn idle(runner: &Runner) -> bool {
    matches!(runner.state, RunnerState::Starting | RunnerState::Ready)
}

pub(crate) fn warm_candidate(snapshot: &Snapshot, runner: &Runner) -> bool {
    idle(runner)
        && label_for(snapshot, runner).is_some_and(|label| {
            runner.label.0 == warm_label(label.platform) && label.own_machines_first()
        })
}

pub(crate) fn label_for<'a>(snapshot: &'a Snapshot, runner: &Runner) -> Option<&'a Label> {
    snapshot.labels.iter().find(|label| {
        runner
            .target
            .as_ref()
            .is_some_and(|target| label.targets.contains(target))
    })
}
