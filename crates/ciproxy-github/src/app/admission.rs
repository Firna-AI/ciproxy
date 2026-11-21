//! Initial Add approval applies only when no retained admission exists.

use ciproxy_store_interface::installation_links::OrganizationActivation;

use super::InstallationDecision;

impl InstallationDecision {
    pub(super) fn initial_activation(self) -> OrganizationActivation {
        match self {
            Self::Target {
                admin_approved,
                billing_hold,
                ..
            } => OrganizationActivation {
                billing_hold,
                admin_approval_hold: !admin_approved,
                ..Default::default()
            },
            Self::Direct => OrganizationActivation::default(),
            Self::AwaitingAssignment => OrganizationActivation {
                admin_approval_hold: true,
                ..Default::default()
            },
        }
    }
}
