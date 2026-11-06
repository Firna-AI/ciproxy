//! Restored cache sessions retain original destination and permission authority.

use ciproxy_protocol::guest::JobDecision;
use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::{ControlMessage, JobPreparationOutcome};
use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use ciproxy_protocol::identifiers::{CommandId, JobId, RunnerId};
use ciproxy_protocol::jobs::{JobScope, UnknownJobScopeReason};
use ciproxy_protocol::runtime::{CachePermissions, RuntimeCredentials};
use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::GuestProvenance;
use crate::boundaries::{CacheMock, StoreMock, VmsMock};
use crate::cache::HttpCache;
use crate::cache_admin::{AdministrationMock, Credential};
use crate::controller::Agent;
use crate::fixtures::{controller, trusted_facts, vm};
use crate::state::MachineState;

pub(super) fn original() -> RuntimeCredentials {
    RuntimeCredentials {
        docker_data_bytes: 17,
        proxy_url: "http://bridge:8230/".into(),
        github_token: Secret::new("original-fixture-token"),
        cache_url: Secret::new("https://cache.example/private/path?secret=query"),
        results_url: Secret::new("https://results.example/original/path"),
        permissions: CachePermissions {
            read: true,
            write: false,
        },
    }
}

pub(super) fn starting(runtime: RuntimeCredentials) -> GuestToAgent {
    GuestToAgent::JobStarting {
        runner_id: RunnerId("job".into()),
        job_id: JobId("job-1".into()),
        runtime,
    }
}

pub(super) async fn sealed_state(runtime: RuntimeCredentials) -> MachineState {
    let agent = controller(
        MachineState {
            runners: vec![vm("job", false)],
            ..MachineState::default()
        },
        Unimock::new((
            StoreMock::save
                .each_call(matching!(_))
                .answers(&|_, _| Ok(())),
            CacheMock::handoff
                .some_call(matching!(_, _, _))
                .returns(Ok(())),
        )),
    );
    agent
        .guest(
            GuestProvenance::InitialConfiguration,
            "job",
            starting(runtime),
        )
        .await
        .unwrap();
    agent.state.into_inner()
}

pub(super) fn facts() -> ControlMessage {
    ControlMessage::JobPreparation {
        command_id: CommandId("facts".into()),
        runner_id: RunnerId("job".into()),
        job_id: JobId("job-1".into()),
        outcome: JobPreparationOutcome::TrustedFacts {
            facts: Box::new(trusted_facts()),
        },
    }
}

#[tokio::test]
async fn restored_handoff_cannot_replace_origins_or_add_an_absent_endpoint() {
    for (add_absent, unsealed) in [(false, false), (true, false), (false, true)] {
        let mut runtime = original();
        if add_absent {
            runtime.cache_url = Secret::new("");
        }
        let state = if unsealed {
            MachineState {
                runners: vec![vm("job", true)],
                ..MachineState::default()
            }
        } else {
            sealed_state(runtime.clone()).await
        };
        if add_absent {
            runtime.cache_url = Secret::new("http://169.254.169.254/cache");
        } else if !unsealed {
            runtime.results_url = Secret::new("http://169.254.169.254/results");
        }
        let registrations = Arc::new(Mutex::new(Vec::new()));
        let registered = registrations.clone();
        let recoveries = Arc::new(Mutex::new(Vec::new()));
        let recovered = recoveries.clone();
        let preparations = Arc::new(Mutex::new(Vec::new()));
        let prepared = preparations.clone();
        let cache_mock = Arc::new(
            Unimock::new((
                AdministrationMock::fallback
                    .each_call(matching!(_))
                    .answers(&|_, _| Ok(())),
                AdministrationMock::revoke
                    .each_call(matching!(_))
                    .answers(&|_, _| Ok(())),
                AdministrationMock::recover.stub(|each| {
                    each.call(matching!(_))
                        .answers_arc(Arc::new(move |_, registration| {
                            recovered.lock().unwrap().push(registration.clone());
                            Ok(None)
                        }))
                        .at_least_times(0);
                }),
                AdministrationMock::register.stub(|each| {
                    each.call(matching!(_))
                        .answers_arc(Arc::new(move |_, registration| {
                            registered.lock().unwrap().push(registration);
                            Ok(Credential {
                                token: Secret::new("fixture-session"),
                                scope: JobScope::Unknown {
                                    reason: UnknownJobScopeReason::MissingToken,
                                },
                            })
                        }))
                        .at_least_times(0);
                }),
            ))
            .no_verify_in_drop(),
        );
        let mut restored = controller(
            state,
            Unimock::new((
                StoreMock::save
                    .each_call(matching!(_))
                    .answers(&|_, _| Ok(())),
                VmsMock::send
                    .some_call(matching!(_, _))
                    .answers_arc(Arc::new(move |_, _, message| {
                        prepared.lock().unwrap().push(message.clone());
                        Ok(())
                    })),
            )),
        );
        restored.cache = Arc::new(HttpCache::with_boundaries(
            cache_mock.clone(),
            cache_mock.clone(),
            cache_mock,
        ));
        restored
            .guest(
                GuestProvenance::InitialConfiguration,
                "job",
                starting(runtime),
            )
            .await
            .unwrap();
        restored.control(facts()).await.unwrap();
        assert_eq!(*recoveries.lock().unwrap(), Vec::new());
        assert_eq!(*registrations.lock().unwrap(), Vec::new());
        assert_eq!(
            *preparations.lock().unwrap(),
            vec![
                AgentToGuest::PrepareJob {
                    job_id: JobId("job-1".into()),
                    decision: JobDecision::GitHubOnly
                };
                2
            ]
        );
    }
}

#[tokio::test]
async fn restored_same_origins_can_register_after_sidecar_loss_with_original_rights() {
    let mut state = sealed_state(original()).await;
    state.runners[0].job_facts = Some(trusted_facts());
    let mut replay = original();
    replay.github_token = Secret::new("replayed-fixture-token");
    replay.cache_url = Secret::new("https://cache.example:443/another/path");
    replay.permissions.write = true;
    let registrations = Arc::new(Mutex::new(Vec::new()));
    let registered = registrations.clone();
    let cache = Arc::new(Unimock::new((
        AdministrationMock::recover
            .some_call(matching!(_))
            .returns(Ok(None)),
        AdministrationMock::register
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, registration| {
                registered.lock().unwrap().push(registration);
                Ok(Credential {
                    token: Secret::new("fixture-session"),
                    scope: JobScope::Unknown {
                        reason: UnknownJobScopeReason::MissingToken,
                    },
                })
            })),
    )));
    let mut restored = controller(state, Unimock::new((
        StoreMock::save.each_call(matching!(_)).answers(&|_, _| Ok(())),
        VmsMock::send.some_call(matching!((_, message) if matches!(message, AgentToGuest::PrepareJob { decision: JobDecision::MachineCache { .. }, .. }))).returns(Ok(())),
    )));
    restored.cache = Arc::new(HttpCache::with_boundaries(
        cache.clone(),
        cache.clone(),
        cache,
    ));
    restored.control_connection(false).await.unwrap();
    restored
        .guest(GuestProvenance::Recovery, "job", starting(replay))
        .await
        .unwrap();
    let registrations = registrations.lock().unwrap();
    assert_eq!(registrations.len(), 1);
    assert_eq!(
        registrations[0].permissions,
        CachePermissions {
            read: true,
            write: false
        }
    );
    assert_eq!(
        registrations[0].cache_url,
        Secret::new("https://cache.example:443/another/path")
    );
}

#[tokio::test]
async fn saved_authority_requires_its_key_and_contains_only_non_secret_origins() {
    let state = sealed_state(original()).await;
    let encoded = serde_json::to_string(&state).unwrap();
    assert!(encoded.contains("cache.example"));
    assert!(encoded.contains("results.example"));
    for secret in [
        "original-fixture-token",
        "private/path",
        "secret=query",
        "original/path",
    ] {
        assert!(!encoded.contains(secret));
    }
    let mut missing = serde_json::to_value(&state).unwrap();
    missing["runners"][0]
        .as_object_mut()
        .unwrap()
        .remove("cache_authority");
    let error = serde_json::from_value::<MachineState>(missing).unwrap_err();
    assert_eq!(error.classify(), serde_json::error::Category::Data);
    let mut old_seal = serde_json::to_value(&state).unwrap();
    old_seal["runners"][0]["cache_authority"]["Sealed"]
        .as_object_mut()
        .unwrap()
        .remove("source_gate");
    let decoded = serde_json::from_value::<MachineState>(old_seal);
    assert_eq!(
        decoded.map_err(|error| error.classify()),
        Err(serde_json::error::Category::Data)
    );
    assert_eq!(
        serde_json::from_str::<MachineState>(&encoded).unwrap(),
        state
    );
}
