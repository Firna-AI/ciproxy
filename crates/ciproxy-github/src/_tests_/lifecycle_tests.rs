//! Removal and suspension deliver narrow idempotent lifecycle instructions.

use ciproxy_store_interface::organizations::AppState;
use unimock::{MockFn, Unimock, matching};

use crate::app::webhook::InstallationAction;
use crate::app::{GitHubApp, InstallationOutcome, LifecycleEvent, LifecycleOutputMock};
use crate::persistence::installation::InstallationStoreMock;
use crate::rest::GitHubRestMock;
use crate::runtime::RuntimeMock;
use crate::scale_sets::{ScaleSetApi, ScaleSetApiMock, ScaleSetFactoryMock};
use std::sync::Arc;

use super::support::{app, at, context, event, labels, organization, repositories};

#[tokio::test]
async fn removal_revokes_store_state_before_last_workspace_cleanup_delivery() {
    let mut removed = organization();
    removed.app_state = AppState::Removed;
    let service = app(Unimock::new((
        InstallationStoreMock::by_github_id
            .some_call(matching!(17))
            .returns(Ok(vec![removed.clone()])),
        InstallationStoreMock::by_installation
            .some_call(matching!(19))
            .returns(Ok(Some(organization()))),
        RuntimeMock::now.some_call(matching!()).returns(at()),
        InstallationStoreMock::remove
            .next_call(matching!(_, _))
            .returns(Ok(())),
        InstallationStoreMock::has_organizations
            .some_call(matching!(_))
            .returns(Ok(false)),
        LifecycleOutputMock::changed
            .next_call(matching!(_))
            .answers(&|_, event| {
                let mut expected = organization();
                expected.app_state = AppState::Removed;
                assert_eq!(
                    event,
                    LifecycleEvent::Removed {
                        organization: expected,
                        last_organization: true
                    }
                );
                Ok(())
            }),
    )));
    assert_eq!(
        service
            .installation(event(InstallationAction::Deleted))
            .await
            .unwrap(),
        InstallationOutcome::Organization(removed)
    );
    let outcome = service
        .installation(event(InstallationAction::Created))
        .await
        .unwrap();
    let mut expected = organization();
    expected.app_state = AppState::Removed;
    assert_eq!(outcome, InstallationOutcome::Organization(expected));
}

#[tokio::test]
async fn suspension_stops_idle_organization_capacity_without_revoking_workspace() {
    let mut suspended = organization();
    suspended.app_state = AppState::Suspended;
    let service = app(Unimock::new((
        InstallationStoreMock::by_github_id
            .some_call(matching!(17))
            .returns(Ok(vec![suspended.clone()])),
        InstallationStoreMock::by_installation
            .some_call(matching!(19))
            .returns(Ok(Some(organization()))),
        InstallationStoreMock::suspend
            .some_call(matching!(_, true))
            .returns(Ok(())),
        LifecycleOutputMock::changed
            .some_call(matching!(_))
            .answers(&|_, event| {
                let mut expected = organization();
                expected.app_state = AppState::Suspended;
                assert_eq!(event, LifecycleEvent::Suspended(expected));
                Ok(())
            }),
    )));
    assert_eq!(
        service
            .installation(event(InstallationAction::Suspend))
            .await
            .unwrap(),
        InstallationOutcome::Organization(suspended)
    );
    let outcome = service
        .installation(event(InstallationAction::Created))
        .await
        .unwrap();
    let mut expected = organization();
    expected.app_state = AppState::Suspended;
    assert_eq!(outcome, InstallationOutcome::Organization(expected));
}

#[tokio::test]
async fn unsuspend_reconciles_access_and_resumes_organization_capacity() {
    let mut suspended = organization();
    suspended.app_state = AppState::Suspended;
    let api: Arc<dyn ScaleSetApi> = Arc::new(Unimock::new(
        ScaleSetApiMock::ensure
            .some_call(matching!("ciproxy", 47))
            .returns(Ok(53)),
    ));
    let service = app(Unimock::new((
        InstallationStoreMock::by_installation
            .some_call(matching!(19))
            .returns(Ok(Some(suspended))),
        InstallationStoreMock::suspend
            .some_call(matching!(_, false))
            .returns(Ok(())),
        GitHubRestMock::repositories
            .some_call(matching!(19))
            .returns(Ok(repositories())),
        GitHubRestMock::restrict_runner_group
            .some_call(matching!(_, _, _, _))
            .returns(Ok(47)),
        InstallationStoreMock::save_repositories
            .some_call(matching!(_, _))
            .returns(Ok(())),
        InstallationStoreMock::scale_sets
            .some_call(matching!(_))
            .returns(Ok(vec![context().scale_set])),
        ScaleSetFactoryMock::for_organization
            .some_call(matching!(_))
            .returns(Ok(api)),
        InstallationStoreMock::labels
            .some_call(matching!(_))
            .returns(Ok(vec![labels().remove(0)])),
        LifecycleOutputMock::changed
            .some_call(matching!(_))
            .answers(&|_, event| {
                assert_eq!(event, LifecycleEvent::Active(organization()));
                Ok(())
            }),
    )));
    assert_eq!(
        service
            .installation(event(InstallationAction::Unsuspend))
            .await
            .unwrap(),
        InstallationOutcome::Organization(organization())
    );
}
