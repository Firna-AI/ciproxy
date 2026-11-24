//! Retire obsolete idle registrations while preserving every running job.

use std::time::Duration;

use crate::actions::{Action, StartReason, StopReason};
use crate::provider::{
    ProviderAccount, ProviderCheck, ProviderDispatch, ProviderId, ProviderRunner, Refusal,
    StartBound,
};
use crate::size_policy::allows_provider_runner;
use crate::snapshot::{Place, Runner, RunnerState, Snapshot, WaitingJob};
use crate::state::{Plan, idle, label_for, warm_candidate};

const PROVIDER_READY_DEADLINE: Duration = Duration::from_secs(600);

pub(crate) fn retire_obsolete(snapshot: &Snapshot, plan: &mut Plan) {
    for machine in &snapshot.machines {
        for runner in &machine.runners {
            let reason = obsolete(snapshot, runner, Place::OwnMachines).or_else(|| {
                (machine.draining && idle(runner)).then_some(StopReason::MachineDraining)
            });
            if let Some(reason) = reason {
                plan.stop(runner, reason);
            }
        }
    }
    for sandbox in snapshot.sandboxes.iter().filter(|sandbox| sandbox.marked) {
        let reason = match &sandbox.runner {
            Some(retained) => obsolete(
                snapshot,
                &retained.runner,
                Place::Provider(sandbox.provider),
            )
            .or_else(|| {
                let label = label_for(snapshot, &retained.runner)?;
                (idle(&retained.runner)
                    && !allows_provider_runner(snapshot, label, sandbox.provider, retained))
                .then_some(StopReason::SizeNoLongerAllowed)
            })
            .or_else(|| {
                let deadline = retained
                    .launched_at?
                    .saturating_add(PROVIDER_READY_DEADLINE);
                (idle(&retained.runner) && snapshot.now >= deadline)
                    .then_some(StopReason::ReadyDeadlinePassed)
            }),
            None => Some(StopReason::OrphanSandbox),
        };
        if let Some(reason) = reason {
            plan.delete(sandbox, reason);
        }
    }
}

fn obsolete(snapshot: &Snapshot, runner: &Runner, place: Place) -> Option<StopReason> {
    if runner.state == RunnerState::Exited {
        return Some(StopReason::RunnerExited);
    }
    if !idle(runner) {
        return None;
    }
    if !snapshot.organizations.contains(&runner.organization_id) {
        return Some(StopReason::OrganizationRemoved);
    }
    let Some(label) = label_for(snapshot, runner) else {
        return Some(StopReason::LabelChanged);
    };
    if label.size != runner.requested_size {
        return Some(StopReason::LabelChanged);
    }
    if !label.allows(place) {
        return Some(StopReason::DestinationDisabled);
    }
    None
}

pub(crate) fn claim_existing(snapshot: &Snapshot, plan: &mut Plan, job: &WaitingJob) -> bool {
    for phase in [RunnerState::Ready, RunnerState::Starting] {
        for machine in &snapshot.machines {
            for runner in &machine.runners {
                if runner.state == phase
                    && (machine.connected || phase == RunnerState::Ready)
                    && can_claim(plan, runner, job)
                {
                    plan.claimed.insert(runner.id.clone());
                    return true;
                }
            }
        }
        for sandbox in snapshot.sandboxes.iter().filter(|sandbox| sandbox.marked) {
            if let Some(retained) = &sandbox.runner
                && retained.runner.state == phase
                && can_claim(plan, &retained.runner, job)
                && retained.dispatch == ProviderDispatch::Accepted
            {
                plan.claimed.insert(retained.runner.id.clone());
                return true;
            }
        }
    }
    false
}

pub(crate) fn claim_unsent(
    snapshot: &Snapshot,
    plan: &mut Plan,
    job: &WaitingJob,
    provider: &ProviderAccount,
) -> bool {
    for (id, retained, bound) in unsent_runners(snapshot) {
        if id == provider.id
            && can_claim(plan, &retained.runner, job)
            && admit_provider_start(snapshot, plan, provider, bound)
        {
            let runner_id = retained.runner.id.clone();
            plan.claimed.insert(runner_id.clone());
            if provider.check == ProviderCheck::Current {
                plan.actions.push(Action::ResumeProviderStart { runner_id });
            }
            return true;
        }
    }
    false
}

/// Apply the same start admission to saved and new provider deliveries.
pub(crate) fn admit_provider_start(
    snapshot: &Snapshot,
    plan: &mut Plan,
    provider: &ProviderAccount,
    bound: Option<StartBound>,
) -> bool {
    snapshot.overflow_enabled
        && provider.connected_geometry().is_some()
        && !provider.facts.refusals.iter().any(|refusal| match refusal {
            Refusal::RetryAfter { not_before } => snapshot.now < *not_before,
            Refusal::RejectedKey | Refusal::BillingRequired => true,
            Refusal::SizeNotOnPlan(_) => false,
        })
        && bound.is_some_and(|bound| match provider.check {
            ProviderCheck::InProgress => plan.can_reserve(provider.id, bound),
            ProviderCheck::Current => plan.reserve_provider(provider.id, bound),
            ProviderCheck::Failed => false,
        })
}

fn unsent_runners(
    snapshot: &Snapshot,
) -> impl Iterator<Item = (ProviderId, &ProviderRunner, Option<StartBound>)> {
    snapshot
        .sandboxes
        .iter()
        .filter(|sandbox| sandbox.marked)
        .filter_map(|sandbox| {
            let retained = sandbox.runner.as_ref()?;
            let ProviderDispatch::Unsent(bound) = retained.dispatch else {
                return None;
            };
            Some((sandbox.provider, retained, bound))
        })
}

/// Pair replacements only after all jobs have claimed the unsent starts they need.
pub(crate) fn with_unsent_replacements(snapshot: &Snapshot, plan: Plan) -> Vec<Action> {
    let mut claimed = plan.claimed;
    plan.actions
        .into_iter()
        .map(|action| {
            let Action::StartRunner(start) = action else {
                return action;
            };
            if start.reason != StartReason::WarmPool
                && let Some((_, retained, _)) = unsent_runners(snapshot).find(|(_, retained, _)| {
                    idle(&retained.runner)
                        && retained.runner.target.as_ref() == Some(&start.target)
                        && claimed.insert(retained.runner.id.clone())
                })
            {
                Action::ReplaceUnsentStart {
                    runner_id: retained.runner.id.clone(),
                    start,
                }
            } else {
                Action::StartRunner(start)
            }
        })
        .collect()
}

fn can_claim(plan: &Plan, runner: &Runner, job: &WaitingJob) -> bool {
    idle(runner)
        && runner.target.as_ref() == Some(&job.target)
        && !plan.retiring.contains(&runner.id)
        && !plan.claimed.contains(&runner.id)
}

pub(crate) fn retire_unneeded(snapshot: &Snapshot, plan: &mut Plan) {
    for machine in &snapshot.machines {
        for runner in &machine.runners {
            if idle(runner)
                && !warm_candidate(snapshot, runner)
                && !plan.claimed.contains(&runner.id)
            {
                plan.stop(runner, StopReason::NoWaitingJob);
            }
        }
    }
    for sandbox in snapshot.sandboxes.iter().filter(|sandbox| sandbox.marked) {
        if let Some(retained) = &sandbox.runner
            && idle(&retained.runner)
            && !plan.claimed.contains(&retained.runner.id)
        {
            plan.delete(sandbox, StopReason::NoWaitingJob);
        }
    }
}
