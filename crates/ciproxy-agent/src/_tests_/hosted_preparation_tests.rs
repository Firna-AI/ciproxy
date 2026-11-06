//! Hosted GitHub-only decisions preserve exact scope and final cache authority.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::{
    AgentMessage, CommandFailure, CommandOutcome, ControlMessage, JobPreparationOutcome,
};
use ciproxy_protocol::cache::CacheConnection;
use ciproxy_protocol::guest::{AgentToGuest, JobDecision, JobPreparation};
use ciproxy_protocol::identifiers::{CommandId, JobId, OrganizationId, RunnerId};
use ciproxy_protocol::jobs::{JobScope, UnknownJobScopeReason};
use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, GuestProvenance, StoreMock, VmsMock};
use crate::cache_authority::CacheAuthority;
use crate::cache_authority_tests::{facts, original, sealed_state, starting};
use crate::controller::Agent;
use crate::fixtures::{controller, trusted_facts, vm};
use crate::state::{MachineState, Mode, PreparationState};

fn github_only(runner: &str, organization: &str, job: &str) -> ControlMessage {
    ControlMessage::JobPreparation {
        command_id: CommandId("github-only".into()),
        runner_id: RunnerId(runner.into()),
        job_id: JobId(job.into()),
        outcome: JobPreparationOutcome::GitHubOnly {
            organization_id: OrganizationId(organization.into()),
        },
    }
}

fn result(outcome: CommandOutcome) -> Option<AgentMessage> {
    Some(AgentMessage::CommandResult {
        command_id: CommandId("github-only".into()),
        outcome,
    })
}

#[tokio::test]
async fn hosted_github_only_is_durable_before_send_and_later_facts_cannot_restore_cache() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let saved = events.clone();
    let sent = events.clone();
    let durable = Arc::new(Mutex::new(None));
    let persisted = durable.clone();
    let preparations = Arc::new(Mutex::new(0));
    let prepared = preparations.clone();
    let mock = Unimock::new((
        StoreMock::save
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, state| {
                *persisted.lock().unwrap() = Some(state.clone());
                saved.lock().unwrap().push("save");
                Ok(())
            })),
        CacheMock::handoff
            .some_call(matching!(_, _, _))
            .returns(Ok(())),
        CacheMock::prepare.stub(|each| {
            each.call(matching!(_, _))
                .answers_arc(Arc::new(move |_, _, _| {
                    *prepared.lock().unwrap() += 1;
                    Ok(None)
                }))
                .at_least_times(0);
        }),
        VmsMock::send
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, record, message| {
                assert_eq!(record.cache_authority, CacheAuthority::GitHubOnly);
                assert_eq!(
                    message,
                    &AgentToGuest::PrepareJob {
                        job_id: JobId("job-1".into()),
                        decision: JobDecision::GitHubOnly,
                    }
                );
                sent.lock().unwrap().push("send");
                Ok(())
            })),
    ))
    .no_verify_in_drop();
    let agent = controller(
        MachineState {
            runners: vec![vm("job", false)],
            ..MachineState::default()
        },
        mock.clone(),
    );
    agent
        .guest(
            GuestProvenance::InitialConfiguration,
            "job",
            starting(original()),
        )
        .await
        .unwrap();
    assert_eq!(agent.state.lock().await.mode, Mode::Active);
    events.lock().unwrap().clear();
    assert_eq!(
        agent
            .control(github_only("job", "org-1", "job-1"))
            .await
            .unwrap(),
        result(CommandOutcome::Applied)
    );
    assert_eq!(*events.lock().unwrap(), vec!["save", "send"]);
    let state = durable.lock().unwrap().clone().unwrap();
    assert_eq!(*agent.state.lock().await, state);
    assert_eq!(state.runners[0].cache_authority, CacheAuthority::GitHubOnly);
    assert_eq!(state.runners[0].job_facts, None);
    assert_eq!(state.runners[0].preparation, PreparationState::Waiting);
    agent
        .control(github_only("job", "org-1", "job-1"))
        .await
        .unwrap();
    assert_eq!(*events.lock().unwrap(), vec!["save", "send", "send"]);
    assert_eq!(*agent.state.lock().await, state);
    let restored: MachineState =
        serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
    let restored = controller(restored, mock.clone());
    restored
        .control(github_only("job", "org-1", "job-1"))
        .await
        .unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        vec!["save", "send", "send", "send"]
    );
    restored.control(facts()).await.unwrap();
    assert_eq!(
        restored.state.lock().await.runners[0].cache_authority,
        CacheAuthority::GitHubOnly
    );
    assert_eq!(*preparations.lock().unwrap(), 0);
}

#[tokio::test]
async fn hosted_github_only_does_not_change_a_job_after_facts_or_preparation() {
    let messages = Arc::new(Mutex::new(Vec::new()));
    let sent = messages.clone();
    let fallbacks = Arc::new(Mutex::new(0));
    let fallback = fallbacks.clone();
    let mock = Unimock::new((
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        CacheMock::prepare
            .some_call(matching!(_, _))
            .answers(&|_, _, _| {
                Ok(Some((
                    CacheConnection {
                        url: Secret::new("http://machine:7887"),
                        token: Secret::new("session"),
                    },
                    JobScope::Unknown {
                        reason: UnknownJobScopeReason::MissingToken,
                    },
                    0,
                )))
            }),
        CacheMock::fallback.stub(|each| {
            each.call(matching!(_))
                .answers_arc(Arc::new(move |_, _| {
                    *fallback.lock().unwrap() += 1;
                    Ok(())
                }))
                .at_least_times(0);
        }),
        VmsMock::send
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, message| {
                sent.lock().unwrap().push(message.clone());
                Ok(())
            })),
    ))
    .no_verify_in_drop();
    let agent = controller(sealed_state(original()).await, mock);
    agent.control(facts()).await.unwrap();
    let prepared = agent.state.lock().await.clone();
    assert!(matches!(
        prepared.runners[0].cache_authority,
        CacheAuthority::Sealed(_)
    ));
    assert_eq!(messages.lock().unwrap().len(), 1);
    assert!(matches!(
        messages.lock().unwrap()[0],
        AgentToGuest::PrepareJob {
            decision: JobDecision::MachineCache { .. },
            ..
        }
    ));
    for (facts, preparation) in [
        (Some(trusted_facts()), PreparationState::Started),
        (Some(trusted_facts()), PreparationState::Waiting),
        (None, PreparationState::Started),
        (
            None,
            PreparationState::Completed(JobPreparation {
                cache_ready: true,
                docker_attached: true,
            }),
        ),
    ] {
        let mut before = prepared.clone();
        before.runners[0].job_facts = facts;
        before.runners[0].preparation = preparation;
        *agent.state.lock().await = before.clone();
        assert_eq!(
            agent
                .control(github_only("job", "org-1", "job-1"))
                .await
                .unwrap(),
            result(CommandOutcome::Applied)
        );
        assert_eq!(*agent.state.lock().await, before);
        assert_eq!(messages.lock().unwrap().len(), 1);
        assert_eq!(*fallbacks.lock().unwrap(), 0);
    }
}

#[tokio::test]
async fn hosted_github_only_requires_exact_runner_organization_and_running_job() {
    let mock = Unimock::new((
        StoreMock::save.stub(|each| {
            each.call(matching!(_))
                .answers(&|_, _| Ok(()))
                .at_least_times(0);
        }),
        VmsMock::send.stub(|each| {
            each.call(matching!(_, _))
                .answers(&|_, _, _| Ok(()))
                .at_least_times(0);
        }),
    ))
    .no_verify_in_drop();
    for (runner, organization, job, running) in [
        ("other", "org-1", "job-1", true),
        ("job", "org-2", "job-1", true),
        ("job", "org-1", "other", true),
        ("job", "org-1", "job-1", false),
    ] {
        let before = MachineState {
            runners: vec![vm("job", running)],
            ..MachineState::default()
        };
        let agent = controller(before.clone(), mock.clone());
        assert_eq!(
            agent
                .control(github_only(runner, organization, job))
                .await
                .unwrap(),
            result(CommandOutcome::Failed {
                reason: CommandFailure::GuestConnection
            })
        );
        assert_eq!(*agent.state.lock().await, before);
    }
}
