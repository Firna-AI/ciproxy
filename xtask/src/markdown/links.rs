//! Pure validation of relative files and GitHub heading fragments.

use std::collections::{BTreeMap, BTreeSet};

use super::files::maintained;
use super::parser::{heading_anchors, link_destinations};
use super::target::{destination_parts, ignored_destination, percent_decode, resolve_target_path};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Problem {
    MissingFile,
    MissingFragment,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Violation {
    pub(crate) source: String,
    pub(crate) destination: String,
    pub(crate) line: usize,
    pub(crate) problem: Problem,
}

pub(crate) fn violations(
    documents: &[(String, String)],
    existing_paths: &[String],
) -> Vec<Violation> {
    let anchors = documents
        .iter()
        .map(|(path, source)| (path.as_str(), heading_anchors(source)))
        .collect::<BTreeMap<_, _>>();
    let paths = existing_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    for (source_path, source) in documents {
        if !maintained(source_path) {
            continue;
        }
        for (destination, line) in link_destinations(source) {
            if ignored_destination(&destination) {
                continue;
            }
            let parts = destination_parts(&destination);
            let target = resolve_target_path(source_path, &percent_decode(parts.path));
            let problem = match target {
                None => Some(Problem::MissingFile),
                Some(target) if !paths.contains(target.as_str()) => Some(Problem::MissingFile),
                Some(target) => parts
                    .fragment
                    .filter(|fragment| !fragment.is_empty() && !parts.is_line_link())
                    .filter(|fragment| {
                        anchors
                            .get(target.as_str())
                            .is_some_and(|ids| !ids.contains(&percent_decode(fragment)))
                    })
                    .map(|_| Problem::MissingFragment),
            };
            if let Some(problem) = problem {
                violations.push(Violation {
                    source: source_path.clone(),
                    destination,
                    line,
                    problem,
                });
            }
        }
    }
    violations
}

#[cfg(test)]
#[path = "_tests_/links_tests.rs"]
mod links_tests;
