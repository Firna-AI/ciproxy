//! A new local release cannot use a Docker grant from the older image.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::{ControlMessage, JobPreparationOutcome};
use ciproxy_protocol::cache::{CacheConnection, DockerBudget};
use ciproxy_protocol::identifiers::{CommandId, JobId, RunnerId};
use ciproxy_protocol::images::ImageReport;
use ciproxy_protocol::jobs::{JobScope, UnknownJobScopeReason};
use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, HostMock, ImagesMock, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::docker::DockerDisksMock;
use crate::fixtures::{controller, hardware, metadata, trusted_facts, vm};
use crate::state::MachineState;

#[tokio::test]
async fn old_image_policy_grants_nothing_after_local_selection_changes() {
    let granted = Arc::new(Mutex::new(Vec::new()));
    let captured = granted.clone();
    let selected = metadata("image-2", 70_000_000_000);
    let newest = selected.clone();
    let mock = Unimock::new((
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        ImagesMock::report
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                Ok(ImageReport {
                    newest: Some(newest.image_id.clone()),
                    available: vec![newest.clone()],
                })
            })),
        CacheMock::prepare
            .each_call(matching!(_, _))
            .answers(&|_, _, _| {
                Ok(Some((
                    CacheConnection {
                        url: Secret::new("http://machine:7887"),
                        token: Secret::new("session"),
                    },
                    JobScope::Unknown {
                        reason: UnknownJobScopeReason::MissingToken,
                    },
                    7,
                )))
            }),
        HostMock::hardware
            .each_call(matching!())
            .answers(&|_| Ok(hardware())),
        DockerDisksMock::prepare
            .each_call(matching!(_, _, _, _, _, _))
            .answers_arc(Arc::new(move |_, _, _, _, _, spare, _| {
                captured.lock().unwrap().push(spare);
                Ok(None)
            })),
        VmsMock::send
            .each_call(matching!(_, _))
            .answers(&|_, _, _| Ok(())),
    ));
    let mut old = vm("old", true);
    old.job_facts = Some(trusted_facts());
    let mut new = vm("new", true);
    new.report.image_id = selected.image_id.clone();
    new.job_facts = Some(trusted_facts());
    let mut agent = controller(
        MachineState {
            runners: vec![old, new],
            ..MachineState::default()
        },
        mock.clone(),
    );
    agent.dockers = Some(Arc::new(mock));
    for (runner, image) in [
        ("old", metadata("image-1", 60_263_759_872)),
        ("new", selected),
    ] {
        agent
            .control(ControlMessage::DockerBudget {
                command_id: CommandId("policy".into()),
                budget: Some(DockerBudget {
                    image,
                    spare_bytes: 100_000_000_000,
                }),
            })
            .await
            .unwrap();
        agent
            .control(ControlMessage::JobPreparation {
                command_id: CommandId("facts".into()),
                runner_id: RunnerId(runner.into()),
                job_id: JobId("job-1".into()),
                outcome: JobPreparationOutcome::TrustedFacts {
                    facts: Box::new(trusted_facts()),
                },
            })
            .await
            .unwrap();
    }
    assert_eq!(*granted.lock().unwrap(), vec![0, 100_000_000_000]);
}
