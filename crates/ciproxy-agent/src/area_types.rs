//! Durable area identity and filesystem facts used before any data mutation.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum Phase {
    Creating,
    Ready,
    Removing,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// The durable identity of the one area owned by this installation.
pub struct AreaRecord {
    pub(crate) version: u32,
    pub(crate) uuid: String,
    pub(crate) source_capacity: u64,
    pub(crate) bytes: u64,
    pub(crate) device: u64,
    pub(crate) inode: u64,
    pub(crate) phase: Phase,
    #[serde(default)]
    pub(crate) image_budget: u64,
    #[serde(default)]
    pub(crate) overhead: u64,
    #[serde(default)]
    pub(crate) growth_from: Option<u64>,
}

#[derive(Clone, Debug)]
pub(crate) struct Backing {
    pub(crate) device: u64,
    pub(crate) inode: u64,
    pub(crate) bytes: u64,
    pub(crate) safe: bool,
}

#[derive(Clone, Debug)]
/// Host capacity and file ownership observed before an area operation.
pub struct Inventory {
    pub(crate) record: Option<AreaRecord>,
    pub(crate) backing: Option<Backing>,
    pub(crate) host_bytes: u64,
    pub(crate) free_bytes: u64,
    pub(crate) area_free: Option<u64>,
    pub(crate) legacy: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Filesystem kind and UUID read from the backing file.
pub struct Filesystem {
    pub(crate) kind: String,
    pub(crate) uuid: String,
}
