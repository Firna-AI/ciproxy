//! The committed archive and its separate uncommitted block versions.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::storage::capture::Captured;

/// One reservation with independent staged versions and active archive ownership.
pub(crate) struct Upload {
    pub(super) key: String,
    pub(super) version: String,
    pub(super) remote: String,
    pub(super) active: Archive,
    /// A present key with no capture must hide the older committed version for Latest.
    pub(super) staged: BTreeMap<String, Option<Arc<Captured>>>,
    pub(super) generation: u64,
}

/// The selected archive form. A missing capture cannot select a different form.
pub(super) enum Archive {
    Whole(Option<Arc<Captured>>),
    Blocks(Vec<CommittedBlock>),
    Ranges(BTreeMap<u64, Arc<Captured>>),
}

pub(super) struct CommittedBlock {
    pub(super) id: String,
    pub(super) capture: Option<Arc<Captured>>,
}
