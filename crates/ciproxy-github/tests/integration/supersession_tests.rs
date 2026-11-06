//! A replacement install completes missed removal before it takes ownership.

use ciproxy_store_interface::github_lifecycle::{LifecycleEvent, RemovalCause};
use std::sync::Arc;

use chrono::Duration;
use ciproxy_github::app::InstallationOutcome;
use ciproxy_github::app::webhook::{Actor, Installation, InstallationAction, InstallationEvent};
use ciproxy_store_interface::boat::BoatAccount;
use ciproxy_store_interface::machines::Machine;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use ciproxy_store_interface::secrets::{EncryptedBoatKey, TokenHash};
use ciproxy_store_interface::store::Store;
use ciproxy_store_interface::tokens::{Token, TokenSubject};
use ciproxy_store_interface::workspaces::Workspace;

use super::app_fixture::{Fixture, at, installed};
use super::database::Database;
use super::support::{id, setup};

fn credentials(store: &Arc<dyn Store>) -> (TokenHash, TokenHash, TokenHash) {
    let machine = Machine {
        id: id(80),
        hostname: "builder".to_owned(),
        processor: "AMD".to_owned(),
        hardware_threads: 8,
        memory_bytes: 32_000_000_000,
        kvm_available: true,
        slots: 3,
        warm_runners: 1,
        draining: false,
        last_contact_at: Some(at()),
        agent_version: "test".to_owned(),
        disks: vec![],
        removed_at: None,
    };
    store.create_machine(id(1), &machine).unwrap();
    let install = TokenHash::new(vec![1]);
    let machine_hash = TokenHash::new(vec![2]);
    let sandbox = TokenHash::new(vec![3]);
    store
        .create_install_token(id(1), id(81), &install, at())
        .unwrap();
    store
        .create_scoped_token(
            &Token {
                id: id(82),
                workspace_id: id(1),
                subject: TokenSubject::Machine(id(80)),
                created_at: at(),
                expires_at: None,
                revoked_at: None,
            },
            &machine_hash,
        )
        .unwrap();
    let account = BoatAccount {
        id: id(83),
        account_name: "cleanup".to_owned(),
        connected_at: at(),
        retired_at: None,
        rejected_at: None,
    };
    store
        .connect_boat(id(1), &account, &EncryptedBoatKey::new(vec![4]), 50_000_000)
        .unwrap();
    let label = store.labels(id(1)).unwrap().remove(0);
    store
        .create_runner(
            id(1),
            &Runner {
                id: id(84),
                organization_id: id(5),
                label_id: label.id,
                scale_set_id: id(6),
                github_id: Some(15),
                state: RunnerState::Busy,
                size: ciproxy_protocol::resources::VmSize {
                    vcpu: label.size.vcpus as u32,
                    memory_bytes: label.size.memory_gb as u64 * 1_000_000_000,
                    disk_bytes: label.size.disk_gb as u64 * 1_000_000_000,
                },
                place: RunnerPlace::Boat {
                    key_id: Some(account.id),
                    sandbox_id: "live-job".to_owned(),
                    size_name: "small".to_owned(),
                },
            },
        )
        .unwrap();
    store
        .create_scoped_token(
            &Token {
                id: id(85),
                workspace_id: id(1),
                subject: TokenSubject::Sandbox {
                    organization_id: id(5),
                    runner_id: id(84),
                },
                created_at: at(),
                expires_at: None,
                revoked_at: None,
            },
            &sandbox,
        )
        .unwrap();
    (install, machine_hash, sandbox)
}

#[tokio::test]
async fn replacement_creation_before_old_deletion_revokes_old_access_before_activation() {
    for explicit_target in [false, true] {
        let database = Database::new();
        setup(&*database.store);
        let (install, machine, sandbox) = credentials(&database.store);
        let fixture = Fixture::new();
        if explicit_target {
            let workspace = Workspace {
                id: id(90),
                name: "target".to_owned(),
                delete_after: None,
            };
            database.store.create_workspace(&workspace).unwrap();
            let mut label = database.store.labels(id(1)).unwrap().remove(0);
            label.id = id(91);
            database.store.create_label(workspace.id, &label).unwrap();
            *fixture.authority.target.lock().unwrap() = Some(workspace.id);
        }
        *fixture.state.current.lock().unwrap() = Some(installed(7, false));
        let app = fixture.app(database.store.clone());
        let created = InstallationEvent {
            action: InstallationAction::Created,
            installation: Installation {
                id: 7,
                account: installed(7, false).account,
            },
            requester: Some(Actor { id: 41 }),
            sender: Actor { id: 99 },
        };
        let result = app.installation(created).await;
        assert!(
            matches!(&result, Ok(InstallationOutcome::Organization(organization)) if organization.installation_id == 7),
            "explicit_target={explicit_target}: {result:?}"
        );
        let current = database
            .store
            .organizations_by_github_id(17)
            .unwrap()
            .into_iter()
            .find(|organization| organization.app_state == AppState::Active)
            .unwrap();
        assert_eq!(
            current.workspace_id,
            if explicit_target { id(90) } else { id(1) }
        );
        assert_eq!(
            database.store.authenticate_token(&install, at()).unwrap(),
            None
        );
        assert_eq!(
            database.store.authenticate_token(&machine, at()).unwrap(),
            None
        );
        assert!(
            database
                .store
                .authenticate_token(&sandbox, at())
                .unwrap()
                .is_some()
        );
        assert_eq!(
            database.store.boat_accounts(id(1)).unwrap()[0].retired_at,
            Some(at())
        );
        assert_eq!(
            database.store.encrypted_boat_key(id(1), id(83)).unwrap(),
            Some(EncryptedBoatKey::new(vec![4]))
        );
        let events = fixture.output.accepted.lock().unwrap().clone();
        assert!(
            matches!(&events[0], LifecycleEvent::Removed { organization, last_organization: true, cause: RemovalCause::Owner } if organization.installation_id == 19)
        );
        assert_eq!(events[1], LifecycleEvent::Active(current.clone()));
        let delayed = InstallationEvent {
            action: InstallationAction::Deleted,
            installation: Installation {
                id: 19,
                account: installed(19, false).account,
            },
            requester: None,
            sender: Actor { id: 99 },
        };
        app.installation(delayed).await.unwrap();
        assert_eq!(
            database
                .store
                .organization(current.workspace_id, current.id)
                .unwrap(),
            Some(current)
        );
        if explicit_target {
            let retained = database.store.organization(id(1), id(5)).unwrap().unwrap();
            assert_eq!(
                (
                    retained.app_state,
                    retained.removed_at,
                    retained.delete_after
                ),
                (
                    AppState::Removed,
                    Some(at()),
                    Some(at() + Duration::days(30))
                )
            );
        }
    }
}
