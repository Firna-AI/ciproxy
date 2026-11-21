//! Selects maintained Markdown for lint and link checks.

use crate::files::ignored_path;

pub(crate) fn maintained(path: &str) -> bool {
    if ignored_path(path) || path.rsplit('/').next() == Some("CLAUDE.md") {
        return false;
    }
    path == "AGENTS.md"
        || path.rsplit('/').next() == Some("README.md")
        || (path.ends_with(".md") && (path.starts_with("docs/") || path.starts_with("plans/")))
        || (path.starts_with("ts/site/") && (path.ends_with(".md") || path.ends_with(".mdx")))
}

#[cfg(test)]
#[path = "_tests_/files_tests.rs"]
mod files_tests;
