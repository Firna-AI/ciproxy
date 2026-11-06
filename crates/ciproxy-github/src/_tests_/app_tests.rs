//! Requester identity, atomic defaults, private access, and installation replay.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use ciproxy_store_interface::github_lifecycle::{GithubLifecycleTarget, LifecycleEvent};
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::organizations::RepositoryAccess;
use ciproxy_store_interface::scale_sets::ScaleSet;
use unimock::{MockFn, Unimock, matching};

use crate::app::webhook::{AccountKind, InstallationAction};
use crate::app::{
    GitHubApp, InstallAuthorityMock, InstallationDecision, InstallationOutcome, LifecycleOutputMock,
};
use crate::persistence::installation::InstallationStoreMock;
use crate::persistence::lifecycle::LifecycleStoreMock;
use crate::rest::GitHubRestMock;
use crate::scale_sets::{ScaleSetApi, ScaleSetApiMock, ScaleSetFactoryMock};

use crate::runtime::RuntimeMock;

use super::coordination::{coordination, delivery};

use super::support::{app, event, id, labels, organization, repositories};

#[tokio::test]
async fn requester_creates_named_workspace_with_atomic_defaults_and_private_group() {
    let ids = AtomicU64::new(0);
    let created = Arc::new(Mutex::new(Vec::new()));
    let observed = created.clone();
    let api: Arc<dyn ScaleSetApi> = Arc::new(Unimock::new((
        ScaleSetApiMock::ensure
            .some_call(matching!("ciproxy", 47, None))
            .returns(Ok(53)),
        ScaleSetApiMock::ensure
            .some_call(matching!("ciproxy-4vcpu", 47, None))
            .returns(Ok(59)),
        ScaleSetApiMock::ensure
            .some_call(matching!("ciproxy-8vcpu", 47, None))
            .returns(Ok(61)),
    )));
    let _keep_api = api.clone();
    let service = app(Unimock::new((
        (
            coordination(false),
            RuntimeMock::uuid
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    id(u128::from(ids.fetch_add(1, Ordering::SeqCst)))
                })),
        ),
        InstallationStoreMock::stage
            .some_call(matching!(_))
            .answers(&|_, claim| Ok(claim)),
        InstallAuthorityMock::installation_decision
            .some_call(matching!(41, 17, 19))
            .returns(Ok(InstallationDecision::Direct)),
        InstallationStoreMock::by_github_id
            .some_call(matching!(17))
            .returns(Ok(vec![])),
        LifecycleStoreMock::register
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, registration| {
                assert_eq!(registration.workspace.unwrap().name, "customer");
                assert_eq!(registration.organization, organization());
                assert_eq!(registration.defaults, labels());
                assert_eq!(registration.initial_admin, 41);
                let GithubLifecycleTarget::Active(prepared) = registration.target else {
                    panic!("active preparation");
                };
                assert_eq!(prepared.repositories, repositories());
                for set in prepared.scale_sets {
                    observed.lock().unwrap().push((
                        set.organization_id,
                        set.label_id,
                        set.github_id,
                        set.runner_group_id,
                    ));
                }
                Ok(registration.organization)
            })),
        InstallationStoreMock::pending_admin
            .some_call(matching!(_))
            .returns(Ok(Some(41))),
        InstallAuthorityMock::ensure_initial_admin
            .some_call(matching!(_, 41))
            .returns(Ok(())),
        InstallationStoreMock::confirm_admin
            .some_call(matching!(_))
            .returns(Ok(())),
        GitHubRestMock::repositories
            .some_call(matching!(19))
            .returns(Ok(repositories())),
        GitHubRestMock::restrict_runner_group
            .some_call(matching!(_, _, _, _, _))
            .answers(&|_, installation, login, _, ids, _| {
                assert_eq!(installation, 19);
                assert_eq!(login, "customer");
                assert_eq!(ids, [23]);
                Ok(47)
            }),
        InstallationStoreMock::scale_sets
            .some_call(matching!(_))
            .returns(Ok(vec![])),
        InstallationStoreMock::runner_group
            .some_call(matching!(17))
            .returns(Ok(None)),
        ScaleSetFactoryMock::for_organization
            .some_call(matching!(_))
            .returns(Ok(api)),
        LifecycleOutputMock::changed
            .some_call(matching!(_))
            .returns(Ok(())),
        delivery(LifecycleEvent::Active(organization())),
    )));
    assert_eq!(
        service
            .installation(event(InstallationAction::Created))
            .await
            .unwrap(),
        InstallationOutcome::Organization(organization())
    );
    let mut created = created.lock().unwrap().clone();
    created.sort_by_key(|entry| entry.1);
    assert_eq!(
        created,
        vec![
            (id(5), id(2), 53, 47),
            (id(5), id(3), 59, 47),
            (id(5), id(4), 61, 47)
        ]
    );
}

#[tokio::test]
async fn admin_intent_uses_shared_workspace_and_replay_keeps_label_edits() {
    let mut existing = organization();
    existing.repositories = RepositoryAccess::Selected(vec![23, 29, 101]);
    let mut edited = labels().remove(0);
    edited.size.vcpus = 6;
    let api: Arc<dyn ScaleSetApi> = Arc::new(Unimock::new(
        ScaleSetApiMock::ensure
            .some_call(matching!("ciproxy", 47, Some(53)))
            .returns(Ok(53)),
    ));
    let service = app(Unimock::new((
        coordination(false),
        InstallationStoreMock::stage
            .some_call(matching!(_))
            .answers(&|_, mut claim| {
                claim.user_verified = true;
                claim.workspace_id = Some(id(1));
                Ok(claim)
            }),
        InstallAuthorityMock::installation_decision
            .some_call(matching!(41, 17, 19))
            .returns(Ok(InstallationDecision::Target {
                workspace_id: id(1),
                admin_approved: true,
                billing_hold: false,
            })),
        InstallationStoreMock::by_github_id
            .some_call(matching!(17))
            .returns(Ok(vec![existing.clone()])),
        LifecycleStoreMock::register
            .some_call(matching!(_))
            .answers(&|_, registration| {
                assert_eq!(registration.workspace, None);
                assert_eq!(registration.defaults, Vec::<Label>::new());
                assert_eq!(
                    registration.organization.repositories,
                    RepositoryAccess::Selected(vec![23, 29, 101])
                );
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
            .answers(&|_, installation, login, _, ids, _| {
                assert_eq!(installation, 19);
                assert_eq!(login, "customer");
                assert_eq!(ids, [23]);
                Ok(47)
            }),
        InstallationStoreMock::scale_sets
            .some_call(matching!(_))
            .returns(Ok(vec![ScaleSet {
                id: id(6),
                organization_id: id(5),
                label_id: id(2),
                github_id: 53,
                runner_group_id: 47,
            }])),
        ScaleSetFactoryMock::for_organization
            .some_call(matching!(_))
            .returns(Ok(api)),
        RuntimeMock::uuid.some_call(matching!()).returns(id(99)),
        InstallationStoreMock::activation
            .some_call(matching!(_))
            .returns(Ok(Default::default())),
        InstallationStoreMock::labels
            .some_call(matching!(_))
            .returns(Ok(vec![edited])),
        delivery(LifecycleEvent::Active(existing.clone())),
        LifecycleOutputMock::changed
            .some_call(matching!(_))
            .returns(Ok(())),
    )));
    assert_eq!(
        service
            .installation(event(InstallationAction::Created))
            .await
            .unwrap(),
        InstallationOutcome::Organization(existing)
    );
}

#[tokio::test]
async fn personal_installation_returns_organizations_only_without_creating_resources() {
    let service = app(Unimock::new(()));
    let mut personal = event(InstallationAction::Created);
    personal.installation.account.kind = AccountKind::User;
    assert_eq!(
        service.installation(personal).await.unwrap(),
        InstallationOutcome::OrganizationsOnly {
            installation_id: 19,
            remove_url: "https://github.com/settings/installations/19".to_owned(),
        }
    );
}
