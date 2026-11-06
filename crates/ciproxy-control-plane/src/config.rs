//! Image-release disk facts shared by snapshots and overflow configuration.

use ciproxy_agent::root_sizing::RootSizing;

/// The image producer's manifest field, independent of installed file bytes.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ImageManifest {
    /// Exact logical length of the exported ext4 root disk.
    pub(crate) root_disk_bytes: u64,
}

/// Resolve the explicit override before release metadata and the shared default.
pub(crate) fn root_sizing(
    override_bytes: Option<u64>,
    manifest: Option<ImageManifest>,
) -> RootSizing {
    RootSizing {
        image_root_bytes: override_bytes
            .or_else(|| manifest.map(|image| image.root_disk_bytes))
            .unwrap_or_else(|| RootSizing::default().image_root_bytes),
    }
}

#[cfg(test)]
#[path = "_tests_/config_tests.rs"]
mod config_tests;
