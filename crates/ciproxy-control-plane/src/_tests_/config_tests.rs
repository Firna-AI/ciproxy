//! Configuration precedence reaches the agent's public allocation rule.

use super::{ImageManifest, root_sizing};

#[test]
fn image_release_metadata_sizes_roots_and_an_explicit_override_takes_precedence() {
    let manifest = Some(ImageManifest {
        root_disk_bytes: 63_000_000_000,
    });
    assert_eq!(
        root_sizing(None, manifest).root_bytes(20_000_000_000),
        Some(83_635_855_360)
    );
    assert_eq!(
        root_sizing(Some(59_000_000_000), manifest).root_bytes(20_000_000_000),
        Some(79_635_853_312)
    );
    assert_eq!(
        root_sizing(None, None),
        ciproxy_agent::root_sizing::RootSizing::default()
    );
}
