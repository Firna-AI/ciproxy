//! A second workspace shares the held remote runtime's real PostgreSQL store.

use ciproxy_gateway::types::Session;
use ciproxy_store_interface::gateway::Contact;
use ciproxy_store_interface::github::GithubJob;
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::tokens::{Token, TokenSubject};
use ciproxy_store_interface::workspaces::Workspace;
use uuid::Uuid;

use super::recovery_support::job;
use super::support::{Fixture, at};

pub(super) struct Peer {
    pub(super) workspace: Uuid,
    pub(super) runner: OwnRunner,
    pub(super) job: GithubJob,
    pub(super) session: Session,
}

pub(super) fn peer(fixture: &Fixture) -> Peer {
    let store = &fixture.store;
    let workspace = Uuid::new_v4();
    store
        .create_workspace(&Workspace {
            id: workspace,
            name: "Independent workspace".into(),
            delete_after: None,
        })
        .unwrap();
    let mut organization = store
        .organization(fixture.workspace, fixture.organization)
        .unwrap()
        .unwrap();
    organization.id = Uuid::new_v4();
    organization.workspace_id = workspace;
    organization.github_id = 142;
    organization.installation_id = 143;
    store.install_organization(&organization).unwrap();
    let mut label = store.labels(fixture.workspace).unwrap().remove(0);
    label.id = Uuid::new_v4();
    store.create_label(workspace, &label).unwrap();
    let mut binding = store
        .scale_sets(fixture.workspace, fixture.organization)
        .unwrap()
        .remove(0);
    binding.id = Uuid::new_v4();
    binding.organization_id = organization.id;
    binding.label_id = label.id;
    binding.github_id = 144;
    store.create_scale_set(workspace, &binding).unwrap();
    let mut machine = store
        .machines(fixture.workspace)
        .unwrap()
        .into_iter()
        .find(|machine| machine.id == fixture.machine)
        .unwrap();
    machine.id = Uuid::new_v4();
    machine.warm_runners = 1;
    machine.draining = false;
    store.create_machine(workspace, &machine).unwrap();
    let mut runner = fixture.intent.clone();
    runner.runner.id = Uuid::new_v4();
    runner.runner.organization_id = organization.id;
    runner.runner.label_id = label.id;
    runner.runner.scale_set_id = binding.id;
    runner.runner.github_id = Some(147);
    runner.registration_scale_set_id = Some(144);
    runner.runner.place = ciproxy_store_interface::runners::RunnerPlace::Machine {
        machine_id: machine.id,
        first_slot: 0,
    };
    runner.command_id = Uuid::new_v4();
    store.create_own_runner(workspace, &runner).unwrap();
    let mut contact: Contact = fixture.contact.clone();
    contact.session_id = Uuid::new_v4();
    contact.report.runners[0].runner_id.0 = runner.runner.id.to_string();
    let hash = TokenHash::new(vec![83; 32]);
    store
        .create_scoped_token(
            &Token {
                id: Uuid::new_v4(),
                workspace_id: workspace,
                subject: TokenSubject::Machine(machine.id),
                created_at: at(),
                expires_at: None,
                revoked_at: None,
            },
            &hash,
        )
        .unwrap();
    store
        .connect_machine(workspace, machine.id, &hash, &contact)
        .unwrap()
        .unwrap();
    let mut job = job(fixture);
    job.organization_id = organization.id;
    job.label_id = label.id;
    job.scale_set_id = binding.id;
    job.runner_github_id = Some(147);
    store.save_github_job(workspace, &job, at()).unwrap();
    Peer {
        workspace,
        runner,
        job,
        session: Session {
            id: contact.session_id,
            workspace_id: workspace,
            machine_id: machine.id,
        },
    }
}

pub(super) fn reserve_warm(fixture: &Fixture) {
    fixture
        .store
        .update_runner(
            fixture.workspace,
            fixture.organization,
            fixture.intent.runner.id,
            ciproxy_store_interface::runners::RunnerState::Stopped,
            Some(47),
        )
        .unwrap();
    super::recovery_support::keep_warm(fixture);
    let mut contact = fixture.contact.clone();
    contact.report.runners.clear();
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &contact)
        .unwrap();
}
