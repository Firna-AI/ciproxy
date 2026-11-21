//! Source coverage, exclusions, and the file-length boundary.

use super::{Violation, is_source, violations};

#[test]
fn source_files_over_300_lines_are_reported() {
    let sources = vec![
        ("crates/agent/src/main.rs".to_owned(), "x\n".repeat(301)),
        ("ts/site/src/index.astro".to_owned(), "x\n".repeat(300)),
        (
            "ts/app/node_modules/vendor/index.ts".to_owned(),
            "x\n".repeat(301),
        ),
    ];
    assert_eq!(
        violations(&sources),
        vec![Violation {
            path: "crates/agent/src/main.rs".to_owned(),
            lines: 301,
        }]
    );
}

#[test]
fn source_selection_covers_workspace_sources_and_skips_generated_output() {
    for path in [
        "crates/protocol/src/lib.rs",
        "xtask/src/main.rs",
        "ts/tokens/index.ts",
        "ts/app/src/app/index.tsx",
        "ts/site/src/pages/index.astro",
        "ts/site/src/styles/global.css",
        "ts/site/eslint.config.js",
    ] {
        assert!(is_source(path), "{path}");
    }
    for path in [
        "ts/app/dist/index.js",
        "ts/app/.expo/types/router.d.ts",
        "ts/app/expo-env.d.ts",
        "ts/site/.astro/content.d.ts",
        "ts/site/node_modules/index.js",
        "ts/package-lock.json",
        "docs/mockups/src/render.tsx",
        "target/generated.rs",
    ] {
        assert!(!is_source(path), "{path}");
    }
}
