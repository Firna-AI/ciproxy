//! Reservation, upload capture, and immutable publication of cache entries.

mod assemble;
mod replacement;
mod reserve;
mod transfer;

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::storage::capture::Captured;

pub(crate) struct Upload {
    key: String,
    version: String,
    remote: String,
    whole: Option<Arc<Captured>>,
    blocks: BTreeMap<String, Arc<Captured>>,
    ranges: BTreeMap<u64, Arc<Captured>>,
    order: Vec<String>,
    generation: u64,
}
