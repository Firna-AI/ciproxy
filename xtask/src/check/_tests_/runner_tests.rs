//! Phase failures, keep-going, and Markdown link outcomes without subprocesses
//! or disk access.

use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use unimock::{MockFn, Unimock, matching};

use crate::command::{CommandRunMock, CommandSpec};
use crate::error::Error;
use crate::files::{RepositoryPathsMock, RepositoryReadMock};

use super::{CheckRunner, Phase, WorkspaceCheck};

#[test]
fn failed_phase_stops_before_the_next_command() {
    let failure_status = ExitStatus::from_raw(1 << 8);
    let commands = Unimock::new(
        CommandRunMock
            .some_call(matching!((CommandSpec { program, .. }) if program == "cargo"))
            .returns(Err(Error::CommandFailed {
                program: "cargo".to_owned(),
                status: failure_status,
            })),
    );
    let repository = Unimock::new(());
    let check = WorkspaceCheck {
        commands: &commands,
        repository: &repository,
    };
    let result = check.run(&[], &[], false);
    assert!(matches!(
        result,
        Err(Error::CommandFailed { program, status })
            if program == "cargo" && status == failure_status
    ));
}

#[test]
fn keep_going_runs_later_phases_and_returns_failure() {
    let failure_status = ExitStatus::from_raw(1 << 8);
    let commands = Unimock::new((
        CommandRunMock
            .some_call(matching!((CommandSpec { program, args })
                if program == "cargo" && args.first().is_some_and(|arg| arg == "fmt")))
            .returns(Err(Error::CommandFailed {
                program: "cargo".to_owned(),
                status: failure_status,
            })),
        CommandRunMock
            .some_call(matching!((CommandSpec { program, args })
                if program == "cargo" && args.first().is_some_and(|arg| arg == "clippy")))
            .returns(Ok(())),
    ));
    let repository = Unimock::new(());
    let check = WorkspaceCheck {
        commands: &commands,
        repository: &repository,
    };

    let result = check.run(&[Phase::RustFmt, Phase::RustClippy], &[], true);

    assert!(matches!(result, Err(Error::PhasesFailed { count: 1 })));
}

#[test]
fn heading_fragment_without_a_heading_warns_and_runs_markdownlint() {
    let repository = Unimock::new((
        RepositoryPathsMock
            .some_call(matching!())
            .returns(Ok(vec!["README.md".to_owned(), "docs/setup.md".to_owned()])),
        RepositoryReadMock
            .some_call(matching!("README.md"))
            .returns(Ok("[Setup](docs/setup.md#install)\n".to_owned())),
        RepositoryReadMock
            .some_call(matching!("docs/setup.md"))
            .returns(Ok("# Setup\n".to_owned())),
    ));
    let commands = Unimock::new(
        CommandRunMock
            .some_call(matching!((CommandSpec { program, args })
                if program == "npm" && args.iter().any(|arg| arg == "markdownlint-cli2")))
            .returns(Ok(()))
            .once(),
    );
    let check = WorkspaceCheck {
        commands: &commands,
        repository: &repository,
    };

    let result = check.run(&[Phase::Markdown], &[], false);

    assert!(matches!(result, Ok(())), "{result:?}");
}

#[test]
fn only_links_to_missing_files_fail_the_markdown_phase() {
    let repository = Unimock::new((
        RepositoryPathsMock
            .some_call(matching!())
            .returns(Ok(vec!["README.md".to_owned(), "docs/setup.md".to_owned()])),
        RepositoryReadMock
            .some_call(matching!("README.md"))
            .returns(Ok(
                "[Setup](docs/setup.md#install)\n[Missing](docs/missing.md)\n".to_owned(),
            )),
        RepositoryReadMock
            .some_call(matching!("docs/setup.md"))
            .returns(Ok("# Setup\n".to_owned())),
    ));
    let commands = Unimock::new(());
    let check = WorkspaceCheck {
        commands: &commands,
        repository: &repository,
    };

    let result = check.run(&[Phase::Markdown], &[], false);

    assert!(
        matches!(result, Err(Error::MarkdownLinks { count: 1 })),
        "{result:?}"
    );
}
