//! A retained install reuses its workspace while explicit Admin intent overrides it.

use std::sync::Arc;

use chrono::Duration;
use ciproxy_store_interface::organizations::AppState;
use unimock::{MockFn, Unimock, matching};

use crate::app::webhook::InstallationAction;
use crate::app::{GitHubApp, InstallAuthorityMock, InstallationOutcome, LifecycleOutputMock};
use crate::persistence::installation::InstallationStoreMock;
use crate::rest::GitHubRestMock;
use crate::runtime::RuntimeMock;
use crate::scale_sets::{ScaleSetApi, ScaleSetApiMock, ScaleSetFactoryMock};

use super::coordination::coordination;

use super::support::{app, at, context, event, id, labels, organization, repositories};

#[tokio::test]
async fn retained_reinstall_reuses_workspace_and_preserves_shared_label_configuration() {
    let mut retained = organization();
    retained.installation_id = 13;
    retained.app_state = AppState::Removed;
    retained.removed_at = Some(at() - Duration::days(1));
    retained.delete_after = Some(at() + Duration::days(29));
    let api: Arc<dyn ScaleSetApi> = Arc::new(Unimock::new(
        ScaleSetApiMock::ensure
            .some_call(matching!("ciproxy", 47, Some(53)))
            .returns(Ok(53)),
    ));
    let service = app(Unimock::new((
        coordination(false),
        InstallAuthorityMock::intended_workspace
            .some_call(matching!(41, 17))
            .returns(Ok(None)),
        InstallationStoreMock::by_github_id
            .some_call(matching!(17))
            .returns(Ok(vec![retained])),
        RuntimeMock::now.some_call(matching!()).returns(at()),
        InstallationStoreMock::register
            .some_call(matching!(_))
            .answers(&|_, registration| {
                assert_eq!(registration.workspace, None);
                assert_eq!(registration.defaults.len(), 0);
                assert_eq!(registration.organization, organization());
                Ok(())
            }),
        InstallationStoreMock::pending_admin
            .some_call(matching!(_))
            .returns(Ok(None)),
        GitHubRestMock::repositories
            .some_call(matching!(19))
            .returns(Ok(repositories())),
        GitHubRestMock::restrict_runner_group
            .some_call(matching!(_, _, _, _, _))
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
        InstallationStoreMock::save_scale_set
            .some_call(matching!(_, _))
            .answers(&|_, _, set| Ok(set)),
        LifecycleOutputMock::changed
            .some_call(matching!(_))
            .returns(Ok(())),
    )));
    assert_eq!(
        service
            .installation(event(InstallationAction::Created))
            .await
            .unwrap(),
        InstallationOutcome::Organization(organization())
    );
}

#[tokio::test]
async fn expired_retained_organization_joins_admin_workspace_as_new_registration() {
    let mut expired = organization();
    expired.id = id(70);
    expired.installation_id = 13;
    expired.app_state = AppState::Removed;
    expired.removed_at = Some(at() - Duration::days(30));
    expired.delete_after = Some(at());
    let api: Arc<dyn ScaleSetApi> = Arc::new(Unimock::new(
        ScaleSetApiMock::ensure
            .some_call(matching!("ciproxy", 47, Some(53)))
            .returns(Ok(53)),
    ));
    let service = app(Unimock::new((
        coordination(false),
        InstallAuthorityMock::intended_workspace
            .some_call(matching!(41, 17))
            .returns(Ok(Some(id(1)))),
        InstallationStoreMock::by_github_id
            .some_call(matching!(17))
            .returns(Ok(vec![expired])),
        RuntimeMock::now.some_call(matching!()).returns(at()),
        RuntimeMock::uuid.some_call(matching!()).returns(id(5)),
        InstallationStoreMock::register
            .some_call(matching!(_))
            .answers(&|_, registration| {
                assert_eq!(registration.workspace, None);
                assert_eq!(registration.defaults.len(), 0);
                assert_eq!(registration.organization.workspace_id, id(1));
                assert_eq!(registration.organization.id, id(5));
                Ok(())
            }),
        InstallationStoreMock::pending_admin
            .some_call(matching!(_))
            .returns(Ok(None)),
        GitHubRestMock::repositories
            .some_call(matching!(19))
            .returns(Ok(repositories())),
        GitHubRestMock::restrict_runner_group
            .some_call(matching!(_, _, _, _, _))
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
        InstallationStoreMock::save_scale_set
            .some_call(matching!(_, _))
            .answers(&|_, _, set| Ok(set)),
        LifecycleOutputMock::changed
            .some_call(matching!(_))
            .returns(Ok(())),
    )));
    assert_eq!(
        service
            .installation(event(InstallationAction::Created))
            .await
            .unwrap(),
        InstallationOutcome::Organization(organization())
    );
}
