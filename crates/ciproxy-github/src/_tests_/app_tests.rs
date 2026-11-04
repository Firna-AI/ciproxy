//! Requester identity, atomic defaults, private access, and installation replay.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::organizations::RepositoryAccess;
use ciproxy_store_interface::scale_sets::ScaleSet;
use unimock::{MockFn, Unimock, matching};

use crate::app::webhook::{AccountKind, InstallationAction};
use crate::app::{GitHubApp, InstallAuthorityMock, InstallationOutcome, LifecycleOutputMock};
use crate::persistence::installation::InstallationStoreMock;
use crate::rest::GitHubRestMock;
use crate::scale_sets::{ScaleSetApi, ScaleSetApiMock, ScaleSetFactoryMock};

use crate::runtime::RuntimeMock;

use super::coordination::coordination;

use super::support::{app, event, id, labels, organization, repositories};

#[tokio::test]
async fn requester_creates_named_workspace_with_atomic_defaults_and_private_group() {
    let ids = AtomicU64::new(1);
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
        InstallAuthorityMock::intended_workspace
            .some_call(matching!(41, 17))
            .returns(Ok(None)),
        InstallationStoreMock::by_github_id
            .some_call(matching!(17))
            .returns(Ok(vec![])),
        InstallationStoreMock::register
            .some_call(matching!(_))
            .answers(&|_, registration| {
                assert_eq!(registration.workspace.unwrap().name, "customer");
                assert_eq!(registration.organization, organization());
                assert_eq!(registration.defaults, labels());
                assert_eq!(registration.initial_admin, 41);
                Ok(())
            }),
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
        InstallationStoreMock::save_repositories
            .some_call(matching!(_, _))
            .returns(Ok(())),
        InstallationStoreMock::scale_sets
            .some_call(matching!(_))
            .returns(Ok(vec![])),
        InstallationStoreMock::runner_group
            .some_call(matching!(17))
            .returns(Ok(None)),
        ScaleSetFactoryMock::for_organization
            .some_call(matching!(_))
            .returns(Ok(api)),
        InstallationStoreMock::labels
            .some_call(matching!(_))
            .returns(Ok(labels())),
        InstallationStoreMock::save_scale_set
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, workspace_id, scale_set| {
                assert_eq!(workspace_id, id(1));
                observed.lock().unwrap().push((
                    scale_set.organization_id,
                    scale_set.label_id,
                    scale_set.github_id,
                    scale_set.runner_group_id,
                ));
                Ok(scale_set)
            })),
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
        InstallAuthorityMock::intended_workspace
            .some_call(matching!(41, 17))
            .returns(Ok(Some(id(1)))),
        InstallationStoreMock::by_github_id
            .some_call(matching!(17))
            .returns(Ok(vec![existing.clone()])),
        InstallationStoreMock::register
            .some_call(matching!(_))
            .answers(&|_, registration| {
                assert_eq!(registration.workspace, None);
                assert_eq!(registration.defaults, Vec::<Label>::new());
                assert_eq!(
                    registration.organization.repositories,
                    RepositoryAccess::Selected(vec![23, 29, 101])
                );
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
            .answers(&|_, installation, login, _, ids, _| {
                assert_eq!(installation, 19);
                assert_eq!(login, "customer");
                assert_eq!(ids, [23]);
                Ok(47)
            }),
        InstallationStoreMock::save_repositories
            .some_call(matching!(_, _))
            .returns(Ok(())),
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
        InstallationStoreMock::labels
            .some_call(matching!(_))
            .returns(Ok(vec![edited])),
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
