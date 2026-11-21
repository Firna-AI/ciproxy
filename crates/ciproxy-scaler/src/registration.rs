//! Immutable identity of the GitHub scale set that can give a runner work.

use ciproxy_protocol::identifiers::OrganizationId;

/// An exact organization, label, stored binding, and remote scale-set identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistrationTarget {
    /// Organization registration in the workspace.
    pub organization_id: OrganizationId,
    /// Immutable shared label identity, independent of its name.
    pub label_id: String,
    /// Stable stored organization-label binding identity.
    pub scale_set_id: String,
    /// Exact remote scale set used for registration.
    pub github_id: i64,
}
