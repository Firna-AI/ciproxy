//! Distinct identifiers and names carried across the agent boundary.

use serde::{Deserialize, Serialize};

macro_rules! identifier {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
        #[serde(transparent)]
        pub struct $name(#[doc = "The opaque string carried on the wire."] pub String);
    };
}

identifier!(WorkspaceId, "The ciproxy workspace that owns a machine.");
identifier!(OrganizationId, "The GitHub identifier of one organization.");
identifier!(
    MachineId,
    "The identifier of one own machine registered in a workspace."
);
identifier!(RunnerId, "The identifier of one runner and its fresh VM.");
identifier!(JobId, "The GitHub identifier of one job.");
identifier!(RepositoryId, "The GitHub identifier of one repository.");
identifier!(ImageId, "The release identifier of one job VM image.");
identifier!(
    CommandId,
    "The identifier used to correlate a command and its result."
);
identifier!(
    LabelName,
    "The GitHub runner label that requests a VM size."
);
identifier!(
    GitRef,
    "A full Git ref, such as refs/heads/main or refs/pull/7/merge."
);
identifier!(
    JobEvent,
    "The GitHub event name, kept open to other event names."
);
