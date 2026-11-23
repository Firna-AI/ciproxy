//! Retained jobs for a retired label with a reused name.

use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::runners::Runner;
use uuid::Uuid;

use super::m18_jobs_support::actions;
use super::m18_support::{Pages, at};

pub(super) fn retired_name(pages: &Pages, source: &Runner, template: &Label) {
    let fixture = &pages.fixture;
    let mut label = template.clone();
    label.id = Uuid::new_v4();
    label.name = "reused".into();
    fixture
        .database
        .store
        .create_label(fixture.workspace, &label)
        .unwrap();
    let mut scale = fixture
        .database
        .store
        .scale_sets(fixture.workspace, fixture.organization.id)
        .unwrap()
        .remove(0);
    scale.id = Uuid::new_v4();
    scale.github_id = 901;
    scale.label_id = label.id;
    fixture
        .database
        .store
        .create_scale_set(fixture.workspace, &scale)
        .unwrap();
    let mut retained = source.clone();
    retained.id = Uuid::new_v4();
    retained.github_id = Some(1004);
    retained.label_id = label.id;
    retained.scale_set_id = scale.id;
    fixture
        .database
        .store
        .create_runner(fixture.workspace, &retained)
        .unwrap();
    actions(pages, &retained, 503, "2026-11-05T10:00:00Z");
    fixture
        .database
        .store
        .retire_dashboard_label(fixture.workspace, label.id, at("2026-11-06T12:00:00Z"))
        .unwrap();
    label.id = Uuid::new_v4();
    fixture
        .database
        .store
        .create_label(fixture.workspace, &label)
        .unwrap();
}
