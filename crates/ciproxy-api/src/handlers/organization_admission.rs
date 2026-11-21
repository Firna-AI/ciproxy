//! New-organization quota prediction, separate from retained activation authority.

use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::organizations::{AppState, Organization};

/// Predict a new registration's billing hold from the actual linked count.
pub(super) fn requires_upgrade(tier: Tier, organizations: &[Organization]) -> bool {
    let linked = organizations
        .iter()
        .filter(|organization| organization.app_state != AppState::Removed)
        .count();
    tier == Tier::Free && linked >= 1
}

#[cfg(test)]
#[path = "_tests_/organization_admission_tests.rs"]
mod organization_admission_tests;
