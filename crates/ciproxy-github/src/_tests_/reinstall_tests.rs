//! A retained install reuses its workspace while explicit Admin intent overrides it.

use std::sync::Arc;

use chrono::Duration;
use ciproxy_store_interface::organizations::AppState;
use unimock::{MockFn, Unimock, matching};

use crate::app::webhook::InstallationAction;
use crate::app::{
    GitHubApp, InstallAuthorityMock, InstallationDecision, InstallationOutcome, LifecycleOutputMock,
};
use crate::persistence::installation::InstallationStoreMock;
use crate::persistence::lifecycle::LifecycleStoreMock;
use crate::rest::GitHubRestMock;
use crate::runtime::RuntimeMock;
use crate::scale_sets::{ScaleSetApi, ScaleSetApiMock, ScaleSetFactoryMock};
use ciproxy_store_interface::github_lifecycle::LifecycleEvent;

use super::coordination::{coordination, delivery};

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
        RuntimeMock::uuid.some_call(matching!()).returns(id(99)),
        InstallationStoreMock::associate
            .some_call(matching!(_))
            .answers(&|_, signed| Ok(signed.associate(None, false).unwrap())),
        InstallAuthorityMock::installation_decision
            .some_call(matching!((claim) if claim.requester_id == 41 && claim.github_id == 17 && claim.installation_id == 19))
            .returns(Ok(InstallationDecision::Direct)),
        InstallationStoreMock::by_github_id
            .some_call(matching!(17))
            .returns(Ok(vec![retained])),
        RuntimeMock::now.some_call(matching!()).returns(at()),
        LifecycleStoreMock::register
            .some_call(matching!(_))
            .answers(&|_, registration| {
                assert_eq!(registration.workspace, None);
                assert_eq!(registration.defaults.len(), 0);
                assert_eq!(registration.organization, organization());
                Ok(registration.organization)
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
        InstallationStoreMock::scale_sets
            .some_call(matching!(_))
            .returns(Ok(vec![context().scale_set])),
        ScaleSetFactoryMock::for_organization
            .some_call(matching!(_))
            .returns(Ok(api)),
        InstallationStoreMock::activation
            .some_call(matching!(_))
            .returns(Ok(Default::default())),
        InstallationStoreMock::labels
            .some_call(matching!(_))
            .returns(Ok(vec![labels().remove(0)])),
        delivery(LifecycleEvent::Active(organization())),
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
        InstallationStoreMock::associate
            .some_call(matching!(_))
            .answers(&|_, signed| {
                let mut claim = signed.associate(None, true).unwrap();
                claim.user_verified = true;
                claim.workspace_id = Some(id(1));
                Ok(claim)
            }),
        InstallAuthorityMock::installation_decision
            .some_call(matching!((claim) if claim.requester_id == 41 && claim.github_id == 17 && claim.installation_id == 19))
            .returns(Ok(InstallationDecision::Target {
                workspace_id: id(1),
                admin_approved: true,
                billing_hold: false,
            })),
        InstallationStoreMock::by_github_id
            .some_call(matching!(17))
            .returns(Ok(vec![expired])),
        RuntimeMock::now.some_call(matching!()).returns(at()),
        RuntimeMock::uuid.each_call(matching!()).returns(id(5)),
        LifecycleStoreMock::register
            .some_call(matching!(_))
            .answers(&|_, registration| {
                assert_eq!(registration.workspace, None);
                assert_eq!(registration.defaults.len(), 0);
                assert_eq!(registration.organization.workspace_id, id(1));
                assert_eq!(registration.organization.id, id(5));
                Ok(registration.organization)
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
        InstallationStoreMock::scale_sets
            .some_call(matching!(_))
            .returns(Ok(vec![context().scale_set])),
        ScaleSetFactoryMock::for_organization
            .some_call(matching!(_))
            .returns(Ok(api)),
        InstallationStoreMock::labels
            .some_call(matching!(_))
            .returns(Ok(vec![labels().remove(0)])),
        delivery(LifecycleEvent::Active(organization())),
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
