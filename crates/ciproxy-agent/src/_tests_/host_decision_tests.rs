//! A received job uses GitHub when trusted facts cannot arrive.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::guest::{AgentToGuest, JobDecision};
use ciproxy_protocol::identifiers::{CommandId, OrganizationId, RunnerId};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, GuestProvenance, StoreMock, VmsMock};
use crate::cache_authority::CacheAuthority;
use crate::cache_authority_tests::{facts, original, starting};
use crate::controller::Agent;
use crate::fixtures::{controller, vm};
use crate::state::MachineState;

#[tokio::test]
async fn host_preserves_received_jobs_without_facts_through_disconnect_drain_remove_and_stop() {
    for operation in ["disconnect", "drain", "remove", "stop"] {
        let decisions = Arc::new(Mutex::new(Vec::new()));
        let sent = decisions.clone();
        let registrations = Arc::new(Mutex::new(0));
        let registered = registrations.clone();
        let cache = Arc::new(
            Unimock::new((
                CacheMock::handoff
                    .some_call(matching!(_, _, _))
                    .answers(&|_, _, _, _| Ok(())),
                CacheMock::prepare
                    .each_call(matching!(_, _))
                    .answers_arc(Arc::new(move |_, _, _| {
                        *registered.lock().unwrap() += 1;
                        Ok(None)
                    })),
            ))
            .no_verify_in_drop(),
        );
        let mut agent = controller(
            MachineState {
                runners: vec![vm("job", false)],
                ..MachineState::default()
            },
            Unimock::new((
                StoreMock::save
                    .each_call(matching!(_))
                    .answers(&|_, _| Ok(())),
                VmsMock::send
                    .each_call(matching!(_, _))
                    .answers_arc(Arc::new(move |_, record, message| {
                        assert_eq!(record.cache_authority, CacheAuthority::GitHubOnly);
                        sent.lock().unwrap().push(message.clone());
                        Ok(())
                    })),
            )),
        );
        agent.cache = cache;
        agent
            .guest(
                GuestProvenance::InitialConfiguration,
                "job",
                starting(original()),
            )
            .await
            .unwrap();
        let command_id = CommandId("preserve-job".into());
        match operation {
            "disconnect" => agent.control_connection(false).await.unwrap(),
            "drain" => {
                agent
                    .control(ControlMessage::Drain { command_id })
                    .await
                    .unwrap();
            }
            "remove" => {
                agent.remove().await.unwrap();
            }
            "stop" => {
                agent
                    .control(ControlMessage::StopRunner {
                        command_id,
                        organization_id: OrganizationId("org-1".into()),
                        runner_id: RunnerId("job".into()),
                    })
                    .await
                    .unwrap();
            }
            _ => unreachable!(),
        }
        assert_eq!(decisions.lock().unwrap().len(), 1, "{operation}");
        assert!(matches!(
            &decisions.lock().unwrap()[0],
            AgentToGuest::PrepareJob {
                decision: JobDecision::GitHubOnly,
                ..
            }
        ));
        agent.control_connection(true).await.unwrap();
        agent.control(facts()).await.unwrap();
        assert_eq!(*registrations.lock().unwrap(), 0, "{operation}");
        assert_eq!(
            agent.state.lock().await.runners[0].cache_authority,
            CacheAuthority::GitHubOnly
        );
    }
}
