//! Removed registrations preserve history without consuming new admission capacity.

use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use uuid::Uuid;

use super::requires_upgrade;

#[test]
fn removed_only_history_does_not_require_an_upgrade_for_a_new_organization() {
    let removed = Organization {
        id: Uuid::nil(),
        workspace_id: Uuid::nil(),
        github_id: 100,
        login: "retained-history".into(),
        installation_id: 101,
        app_state: AppState::Removed,
        repositories: RepositoryAccess::AllPrivate,
        removed_at: None,
        delete_after: None,
    };
    assert!(!requires_upgrade(Tier::Free, &[removed]));
}
