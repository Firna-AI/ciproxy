//! Side effects of the gate, behind command and repository boundaries.

use crate::command::{CommandRunner, CommandSpec};
use crate::error::{Error, Result};
use crate::file_length;
use crate::files::Repository;
use crate::markdown::{files, links};

use super::phases::{Phase, commands, selected_phases};

pub(crate) trait CheckRunner {
    fn run(&self, include: &[Phase], exclude: &[Phase], keep_going: bool) -> Result<()>;
}

pub(crate) struct WorkspaceCheck<'a> {
    pub(crate) commands: &'a dyn CommandRunner,
    pub(crate) repository: &'a dyn Repository,
}

impl CheckRunner for WorkspaceCheck<'_> {
    fn run(&self, include: &[Phase], exclude: &[Phase], keep_going: bool) -> Result<()> {
        let mut failures = Vec::new();
        for phase in selected_phases(include, exclude) {
            eprintln!("\n=== {phase:?} ===");
            if let Err(error) = self.run_phase(phase) {
                if !keep_going {
                    return Err(error);
                }
                failures.push((phase, error));
            }
        }
        for (phase, error) in &failures {
            eprintln!("FAILED {phase:?}: {error}");
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(Error::PhasesFailed {
                count: failures.len(),
            })
        }
    }
}

impl WorkspaceCheck<'_> {
    fn run_phase(&self, phase: Phase) -> Result<()> {
        match phase {
            Phase::FileLength => self.check_lengths(),
            Phase::Markdown => self.check_markdown(),
            _ => {
                for command in commands(phase) {
                    self.commands.run(&command)?;
                }
                Ok(())
            }
        }
    }

    fn check_lengths(&self) -> Result<()> {
        let paths = self.repository.paths()?;
        let sources = self.read_selected(&paths, file_length::is_source)?;
        let violations = file_length::violations(&sources);
        for violation in &violations {
            eprintln!("{}: {} lines, maximum 300", violation.path, violation.lines);
        }
        if !violations.is_empty() {
            return Err(Error::FileLength {
                count: violations.len(),
            });
        }
        eprintln!("File length passed for {} sources", sources.len());
        Ok(())
    }

    fn check_markdown(&self) -> Result<()> {
        let paths = self.repository.paths()?;
        let documents = self.read_selected(&paths, |path| {
            (path.ends_with(".md") || path.ends_with(".mdx"))
                && path.rsplit('/').next() != Some("CLAUDE.md")
        })?;
        let violations = links::violations(&documents, &paths);
        for violation in &violations {
            eprintln!(
                "{}:{}: {:?}: {}",
                violation.source, violation.line, violation.problem, violation.destination
            );
        }
        if !violations.is_empty() {
            return Err(Error::MarkdownLinks {
                count: violations.len(),
            });
        }
        let maintained = documents
            .into_iter()
            .filter(|(path, _)| files::maintained(path))
            .map(|(path, _)| path)
            .collect::<Vec<_>>();
        eprintln!("Local links passed for {} documents", maintained.len());
        let mut command = CommandSpec::new(
            "npm",
            &[
                "--prefix",
                "ts",
                "exec",
                "--",
                "markdownlint-cli2",
                "--config",
                "xtask/markdownlint-cli2.jsonc",
            ],
        );
        command.args.extend(maintained);
        self.commands.run(&command)
    }

    fn read_selected(
        &self,
        paths: &[String],
        select: fn(&str) -> bool,
    ) -> Result<Vec<(String, String)>> {
        paths
            .iter()
            .filter(|path| select(path))
            .map(|path| Ok((path.clone(), self.repository.read(path)?)))
            .collect()
    }
}

#[cfg(test)]
#[path = "_tests_/runner_tests.rs"]
mod runner_tests;
