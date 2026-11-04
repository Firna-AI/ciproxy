//! Counted restores retain their measured content origin, including misses.

use crate::cache::{CacheRestoreCounts, CacheSource};

#[test]
fn handled_restore_counts_derive_local_github_mixed_or_no_source() {
    let cases = [
        (
            "cold machine cache",
            0,
            2,
            3_000_000_000,
            Some(CacheSource::Github),
        ),
        (
            "local contents",
            3,
            0,
            2_000_000_000,
            Some(CacheSource::Machine),
        ),
        (
            "mixed contents",
            1,
            1,
            1_000_000_000,
            Some(CacheSource::MachineAndGithub),
        ),
        ("no restored entries", 0, 0, 0, None),
    ];
    for (name, local_entries, github_entries, restored_bytes, expected) in cases {
        let counts = CacheRestoreCounts {
            local_entries,
            github_entries,
            restored_bytes,
        };
        assert_eq!(counts.source(), expected, "{name}");
    }
}
