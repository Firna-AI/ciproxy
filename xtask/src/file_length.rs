//! Pure source selection and the 300-line file limit.

use crate::files::ignored_path;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Violation {
    pub(crate) path: String,
    pub(crate) lines: usize,
}

pub(crate) fn is_source(path: &str) -> bool {
    if ignored_path(path) {
        return false;
    }
    let rust = (path.starts_with("crates/") || path.starts_with("xtask/")) && path.ends_with(".rs");
    let typescript = ["ts/tokens/", "ts/app/", "ts/site/"]
        .iter()
        .any(|root| path.starts_with(root))
        && [
            ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".astro", ".css",
        ]
        .iter()
        .any(|extension| path.ends_with(extension));
    rust || typescript
}

pub(crate) fn violations(sources: &[(String, String)]) -> Vec<Violation> {
    sources
        .iter()
        .filter(|(path, _)| is_source(path))
        .filter_map(|(path, source)| {
            let lines = source.lines().count();
            (lines > 300).then(|| Violation {
                path: path.clone(),
                lines,
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "_tests_/file_length_tests.rs"]
mod file_length_tests;
