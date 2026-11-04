//! Locked listeners preserve assignment metadata and stop before unsafe ACKs.

use std::sync::Arc;

use ciproxy_store_interface::github::{DemandPhase, GithubJob, JobDelivery};
use scaleset::{MessageId, ScaleSetMessage};
use tokio::sync::watch;
use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::listener::{DurableListener, JobOutputMock, ListenerOptions, ScaleSetListener};
use crate::persistence::session::{SessionLease, SessionLeaseMock, SessionStoreMock};
use crate::rest::GitHubRestMock;
use crate::scale_sets::{
    MessageQueue, MessageQueueMock, ScaleSetApi, ScaleSetApiMock, ScaleSetFactoryMock,
};

use super::support::{context, facts, id, target};

fn listener(mock: Unimock) -> DurableListener {
    let mock = Arc::new(mock);
    DurableListener {
        store: mock.clone(),
        rest: mock.clone(),
        scale_sets: mock.clone(),
        output: mock,
    }
}

fn message() -> ScaleSetMessage {
    let mut message = ScaleSetMessage::default();
    message.id = MessageId(7);
    message.jobs = serde_json::from_str(r#"[{
        "messageType":"JobAssigned","jobId":"00000000-0000-0000-0000-000000000071",
        "runnerRequestId":0,"workflowRunId":31,"ownerName":"customer","repositoryName":"private",
        "jobDisplayName":"test","jobWorkflowRef":"shared/reusable/.github/workflows/check.yml@refs/heads/trunk",
        "eventName":"untrusted-message-event","queueTime":"0001-01-01T00:00:00Z",
        "scaleSetAssignTime":"2026-11-04T10:46:21.620395766Z","runnerAssignTime":"0001-01-01T00:00:00Z"
    }]"#).unwrap();
    message
}

fn expected_job() -> GithubJob {
    GithubJob {
        id: id(0x71),
        organization_id: id(5),
        scale_set_id: id(6),
        label_id: id(2),
        actions_job_id: None,
        request_id: 0,
        run_id: 31,
        repository: "customer/private".to_owned(),
        name: "test".to_owned(),
        workflow_ref: "shared/reusable/.github/workflows/check.yml@refs/heads/trunk".to_owned(),
        phase: DemandPhase::Assigned,
        result: None,
        assigned_at: Some("2026-11-04T10:46:21.620395766Z".parse().unwrap()),
        started_at: None,
        completed_at: None,
        runner_github_id: None,
        facts: facts(),
    }
}

fn options() -> ListenerOptions {
    ListenerOptions {
        owner: "instance-1".to_owned(),
        capacity: 2,
        max_polls: Some(1),
    }
}

#[tokio::test]
async fn assigned_job_is_durable_with_api_facts_before_output_and_ack() {
    let service = listener(Unimock::new((
        SessionStoreMock::lock
            .some_call(matching!(_))
            .answers(&|mock, _| Ok(Some(Box::new(mock.clone()) as Box<dyn SessionLease>))),
        SessionLeaseMock::check
            .each_call(matching!())
            .answers(&|_| Ok(())),
        SessionStoreMock::context
            .each_call(matching!(_))
            .answers(&|_, _| Ok(context())),
        ScaleSetFactoryMock::for_organization
            .some_call(matching!(_))
            .answers(&|mock, _| Ok(Arc::new(mock.clone()) as Arc<dyn ScaleSetApi>)),
        ScaleSetApiMock::open
            .some_call(matching!(53, "instance-1"))
            .answers(&|mock, _, _| Ok(Box::new(mock.clone()) as Box<dyn MessageQueue>)),
        MessageQueueMock::poll
            .some_call(matching!(None, 2))
            .returns(Ok(Some(message()))),
        GitHubRestMock::run_facts
            .some_call(matching!(19, "customer/private", 31))
            .returns(Ok(facts())),
        SessionStoreMock::save
            .next_call(matching!(_, _))
            .answers(&|_, workspace_id, job| {
                assert_eq!(workspace_id, id(1));
                assert_eq!(job, expected_job());
                Ok(())
            }),
        SessionStoreMock::pending.stub(|each| {
            each.call(matching!(_))
                .answers(&|_, _| Ok(vec![]))
                .once()
                .then()
                .answers(&|_, _| {
                    Ok(vec![JobDelivery {
                        job: expected_job(),
                        revision: 1,
                    }])
                });
        }),
        JobOutputMock::changed
            .next_call(matching!(_))
            .returns(Ok(())),
        SessionStoreMock::confirm
            .next_call(matching!(_, _, 1))
            .returns(Ok(())),
        MessageQueueMock::acknowledge
            .next_call(matching!(MessageId(7)))
            .returns(Ok(())),
        MessageQueueMock::close
            .some_call(matching!())
            .returns(Ok(())),
    )));
    let (_stop, signal) = watch::channel(false);
    service.run(target(), options(), signal).await.unwrap();
}

#[tokio::test]
async fn failed_storage_keeps_message_unacknowledged_and_closes_session() {
    let service = listener(Unimock::new((
        SessionStoreMock::lock
            .some_call(matching!(_))
            .answers(&|mock, _| Ok(Some(Box::new(mock.clone()) as Box<dyn SessionLease>))),
        SessionLeaseMock::check
            .each_call(matching!())
            .answers(&|_| Ok(())),
        SessionStoreMock::context
            .each_call(matching!(_))
            .answers(&|_, _| Ok(context())),
        ScaleSetFactoryMock::for_organization
            .some_call(matching!(_))
            .answers(&|mock, _| Ok(Arc::new(mock.clone()) as Arc<dyn ScaleSetApi>)),
        ScaleSetApiMock::open
            .some_call(matching!(53, _))
            .answers(&|mock, _, _| Ok(Box::new(mock.clone()) as Box<dyn MessageQueue>)),
        MessageQueueMock::poll
            .some_call(matching!(_, _))
            .returns(Ok(Some(message()))),
        GitHubRestMock::run_facts
            .some_call(matching!(19, "customer/private", 31))
            .returns(Ok(facts())),
        SessionStoreMock::save
            .some_call(matching!(_, _))
            .returns(Err(Error::AccessUnavailable(503))),
        SessionStoreMock::pending
            .some_call(matching!(_))
            .returns(Ok(vec![])),
        MessageQueueMock::close
            .some_call(matching!())
            .returns(Ok(())),
    )));
    let (_stop, signal) = watch::channel(false);
    assert!(matches!(
        service.run(target(), options(), signal).await,
        Err(Error::AccessUnavailable(503))
    ));
}

#[tokio::test]
async fn competing_listener_does_not_open_github_session() {
    let service = listener(Unimock::new(
        SessionStoreMock::lock
            .some_call(matching!(_))
            .returns(Ok(None)),
    ));
    let (_stop, signal) = watch::channel(false);
    assert!(matches!(
        service.run(target(), options(), signal).await,
        Err(Error::SessionOwned)
    ));
}

#[tokio::test]
async fn lost_database_session_stops_processing_and_closes_github_session() {
    let service = listener(Unimock::new((
        SessionStoreMock::lock
            .some_call(matching!(_))
            .answers(&|mock, _| Ok(Some(Box::new(mock.clone()) as Box<dyn SessionLease>))),
        SessionLeaseMock::check.stub(|each| {
            each.call(matching!())
                .answers(&|_| Ok(()))
                .n_times(2)
                .then()
                .answers(&|_| Err(Error::OrganizationInactive));
        }),
        SessionStoreMock::context
            .each_call(matching!(_))
            .answers(&|_, _| Ok(context())),
        ScaleSetFactoryMock::for_organization
            .some_call(matching!(_))
            .answers(&|mock, _| Ok(Arc::new(mock.clone()) as Arc<dyn ScaleSetApi>)),
        ScaleSetApiMock::open
            .some_call(matching!(53, _))
            .answers(&|mock, _, _| Ok(Box::new(mock.clone()) as Box<dyn MessageQueue>)),
        MessageQueueMock::poll
            .some_call(matching!(_, _))
            .returns(Ok(Some(message()))),
        SessionStoreMock::pending
            .some_call(matching!(_))
            .returns(Ok(vec![])),
        MessageQueueMock::close
            .some_call(matching!())
            .returns(Ok(())),
    )));
    let (_stop, signal) = watch::channel(false);
    assert!(matches!(
        service.run(target(), options(), signal).await,
        Err(Error::OrganizationInactive)
    ));
}
