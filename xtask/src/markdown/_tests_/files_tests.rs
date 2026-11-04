//! The maintained Markdown set and exclusions.

use super::maintained;

#[test]
fn maintained_markdown_includes_docs_and_readmes_without_generated_files() {
    for path in [
        "AGENTS.md",
        "README.md",
        "crates/agent/README.md",
        "docs/protocol/agent.md",
        "plans/foundation.md",
        "ts/site/src/content/docs/docs/index.mdx",
    ] {
        assert!(maintained(path), "{path}");
    }
    for path in [
        "CLAUDE.md",
        "docs/CLAUDE.md",
        "ts/node_modules/package/README.md",
        "target/README.md",
        ".context/README.md",
        "docs/mockups/mokly-generated/README.md",
        "ts/site/dist/index.md",
        "ts/site/.astro/content.md",
        "notes.md",
    ] {
        assert!(!maintained(path), "{path}");
    }
}
