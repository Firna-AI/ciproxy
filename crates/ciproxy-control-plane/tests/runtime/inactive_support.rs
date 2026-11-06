//! Running and idle allocations share a machine with an admitted sibling.

use ciproxy_protocol::identifiers::{JobId, OrganizationId, RunnerId};
use ciproxy_protocol::runners;
use ciproxy_store_interface::github::{DemandPhase, Repository};
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runners::RunnerPlace;
use ciproxy_store_interface::scale_sets::ScaleSet;
use uuid::Uuid;

use super::recovery_support::{job, keep_warm, other_organization};
use super::support::{Fixture, at};

pub(super) fn populate(fixture: &mut Fixture) -> (Organization, OwnRunner, OwnRunner) {
    fixture
        .store
        .replace_repositories(
            fixture.workspace,
            fixture.organization,
            &[Repository {
                id: 8,
                full_name: "org/repo".into(),
                private: true,
                default_branch: "main".into(),
            }],
        )
        .unwrap();
    keep_warm(fixture);
    fixture.add_machine();
    let sibling = other_organization(fixture);
    let set = ScaleSet {
        id: Uuid::new_v4(),
        organization_id: sibling.id,
        label_id: fixture.intent.runner.label_id,
        github_id: 92,
        runner_group_id: 93,
    };
    fixture
        .store
        .create_scale_set(fixture.workspace, &set)
        .unwrap();
    let running = additional(
        fixture,
        fixture.organization,
        fixture.intent.runner.scale_set_id,
        1,
        49,
    );
    let mut started = job(fixture);
    started.runner_github_id = Some(49);
    fixture
        .store
        .save_github_job(fixture.workspace, &started, at())
        .unwrap();
    let ready = additional(fixture, sibling.id, set.id, 2, 48);
    let mut queued = job(fixture);
    queued.id = Uuid::new_v4();
    queued.phase = DemandPhase::Assigned;
    queued.runner_github_id = None;
    queued.started_at = None;
    fixture
        .store
        .save_github_job(fixture.workspace, &queued, at())
        .unwrap();
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    fixture.hold_organization();
    (sibling, running, ready)
}

fn additional(
    fixture: &mut Fixture,
    organization: Uuid,
    set: Uuid,
    slot: u64,
    github: i64,
) -> OwnRunner {
    let mut runner = fixture.intent.clone();
    runner.runner.id = Uuid::new_v4();
    runner.runner.organization_id = organization;
    runner.runner.scale_set_id = set;
    runner.runner.github_id = Some(github);
    runner.runner.place = RunnerPlace::Machine {
        machine_id: fixture.machine,
        first_slot: slot as i32,
    };
    runner.slot_indices = vec![slot];
    runner.command_id = Uuid::new_v4();
    runner.acknowledged = false;
    fixture
        .store
        .create_own_runner(fixture.workspace, &runner)
        .unwrap();
    let mut report = fixture.contact.report.runners[0].clone();
    report.runner_id = RunnerId(runner.runner.id.to_string());
    report.organization_id = OrganizationId(if github == 48 { "90" } else { "42" }.into());
    report.slot_indices = vec![slot];
    if github == 49 {
        report.state = runners::RunnerState::Running {
            job_id: JobId("received".into()),
        };
    }
    fixture.contact.report.runners.push(report);
    runner
}
