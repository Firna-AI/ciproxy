//! Shared boundaries for durable runner recovery scenarios.

use std::sync::{Arc, Mutex};

use ciproxy_control_plane::runtime::MachineRuntime;
use ciproxy_gateway::gateway::{MachineGateway, MachineGatewayMock};
use ciproxy_gateway::types::Delivery;
use ciproxy_github::assignments::AssignmentService;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::RunnerRegistrations;
use ciproxy_github::runtime::Runtime;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_store_interface::github::{DemandPhase, GithubJob, RepositoryIdentity, RunFacts};
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::tokens::{Token, TokenSubject};
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::clock::clock;
use super::support::{Fixture, at};

pub(super) fn runtime(
    fixture: &Fixture,
    registrations: Arc<dyn RunnerRegistrations>,
    gateway: Arc<dyn MachineGateway>,
) -> MachineRuntime {
    runtime_with_clock(fixture, registrations, gateway, clock(true))
}

pub(super) fn runtime_with_clock(
    fixture: &Fixture,
    registrations: Arc<dyn RunnerRegistrations>,
    gateway: Arc<dyn MachineGateway>,
    clock: Arc<dyn Runtime>,
) -> MachineRuntime {
    MachineRuntime::new(
        fixture.store.clone(),
        gateway,
        registrations,
        Arc::new(AssignmentService {
            store: Arc::new(StoreAdapter::new(fixture.store.clone(), clock.clone())),
        }),
        clock,
        None,
        Arc::new(Unimock::new(())),
    )
}

pub(super) fn gateway(messages: Arc<Mutex<Vec<ControlMessage>>>) -> Arc<dyn MachineGateway> {
    Arc::new(Unimock::new(
        MachineGatewayMock::send
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, _, message| {
                messages.lock().unwrap().push(message);
                Ok(Delivery::Connected)
            })),
    ))
}

pub(super) fn saved(fixture: &Fixture) -> OwnRunner {
    fixture
        .store
        .own_runners(fixture.workspace)
        .unwrap()
        .remove(0)
}

pub(super) fn keep_warm(fixture: &Fixture) {
    let mut machine = fixture.store.machines(fixture.workspace).unwrap().remove(0);
    machine.draining = false;
    machine.warm_runners = 1;
    fixture
        .store
        .update_machine(fixture.workspace, &machine)
        .unwrap();
}

pub(super) fn reconnect(fixture: &Fixture) {
    let hash = TokenHash::new(vec![71; 32]);
    fixture
        .store
        .create_scoped_token(
            &Token {
                id: Uuid::new_v4(),
                workspace_id: fixture.workspace,
                subject: TokenSubject::Machine(fixture.machine),
                created_at: at(),
                expires_at: None,
                revoked_at: None,
            },
            &hash,
        )
        .unwrap();
    let mut contact = fixture.contact.clone();
    contact.session_id = Uuid::new_v4();
    contact.at = at() + chrono::Duration::seconds(1);
    let machine = fixture
        .store
        .connect_machine(fixture.workspace, fixture.machine, &hash, &contact)
        .unwrap()
        .unwrap();
    assert_eq!(machine.id, fixture.machine);
    assert_eq!(
        fixture
            .store
            .machine_session(fixture.workspace, fixture.machine)
            .unwrap(),
        Some(contact.session_id)
    );
}

pub(super) fn other_organization(fixture: &Fixture) -> Organization {
    let mut other = fixture
        .store
        .organization(fixture.workspace, fixture.organization)
        .unwrap()
        .unwrap();
    other.id = Uuid::new_v4();
    other.github_id = 90;
    other.installation_id = 91;
    other.login = "other".to_owned();
    fixture.store.install_organization(&other).unwrap();
    other
}

pub(super) fn job(fixture: &Fixture) -> GithubJob {
    GithubJob {
        id: Uuid::new_v4(),
        organization_id: fixture.organization,
        scale_set_id: fixture.intent.runner.scale_set_id,
        label_id: fixture.intent.runner.label_id,
        actions_job_id: None,
        request_id: 0,
        run_id: 7,
        repository: "org/repo".to_owned(),
        name: "build".to_owned(),
        workflow_ref: "ci.yml".to_owned(),
        phase: DemandPhase::Started,
        result: None,
        assigned_at: Some(at()),
        started_at: Some(at()),
        completed_at: None,
        runner_github_id: Some(47),
        facts: RunFacts {
            id: 7,
            repository: RepositoryIdentity {
                id: 8,
                full_name: "org/repo".to_owned(),
            },
            event: "push".to_owned(),
            head_branch: Some("main".to_owned()),
            head_sha: "fixture".to_owned(),
            head_repository: None,
        },
    }
}
