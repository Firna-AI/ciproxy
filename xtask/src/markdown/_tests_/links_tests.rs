//! Local link paths, GitHub heading fragments, and URL exclusions.

use super::{Problem, Violation, violations};

#[test]
fn missing_relative_file_reports_its_source_line() {
    let documents = vec![(
        "docs/start.md".to_owned(),
        "# Start\n\n[Setup](../setup.md)\n".to_owned(),
    )];
    assert_eq!(
        violations(&documents, &["docs/start.md".to_owned()]),
        vec![Violation {
            source: "docs/start.md".to_owned(),
            destination: "../setup.md".to_owned(),
            line: 3,
            problem: Problem::MissingFile,
        }]
    );
}

#[test]
fn missing_heading_reports_a_fragment_in_an_existing_file() {
    let documents = vec![
        (
            "README.md".to_owned(),
            "[Setup](docs/setup.md#install)".to_owned(),
        ),
        ("docs/setup.md".to_owned(), "# Setup\n".to_owned()),
    ];
    let paths = documents
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        violations(&documents, &paths),
        vec![Violation {
            source: "README.md".to_owned(),
            destination: "docs/setup.md#install".to_owned(),
            line: 1,
            problem: Problem::MissingFragment,
        }]
    );
}

#[test]
fn valid_links_use_relative_paths_and_github_heading_ids() {
    let documents = vec![
        (
            "docs/start.md".to_owned(),
            "[Read](../README.md?plain=1#use-ciproxy)\n\
             [Again](../README.md#use-ciproxy-1)\n\
             [Space](<setup%20guide.md#install>)\n\
             [Local](#start)\n\
             [Web](https://example.com/missing#heading)\n\
             [Mail](mailto:hello@example.com)\n\
             [Route](/docs/missing/)\n\n# Start\n"
                .to_owned(),
        ),
        (
            "README.md".to_owned(),
            "# Use `ciproxy`!\n\n# Use ciproxy!\n".to_owned(),
        ),
        ("docs/setup guide.md".to_owned(), "## Install\n".to_owned()),
    ];
    let paths = documents
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    assert_eq!(violations(&documents, &paths), Vec::new());
}
