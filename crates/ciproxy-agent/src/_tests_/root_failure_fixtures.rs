//! Ready-only root I/O faults shared by preparation and startup regressions.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use futures_util::FutureExt;
use unimock::{MockFn, Unimock, matching};

use crate::error::{Error, ErrorResultExt, Result};
use crate::layout::Layout;
use crate::process::CommandsMock;
use crate::vm_root::{ReflinkRoot, RootDisk, RootFilesMock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Stage {
    ImageLength,
    Reflink,
    FirstLength,
    Grow,
    FinalLength,
}

impl Stage {
    pub(super) const ALL: [Self; 5] = [
        Self::ImageLength,
        Self::Reflink,
        Self::FirstLength,
        Self::Grow,
        Self::FinalLength,
    ];

    pub(super) fn expected_error(self) -> ErrorFacts {
        match self {
            Self::ImageLength => ErrorFacts::Io(io::ErrorKind::NotFound),
            Self::Reflink => ErrorFacts::HostCommand("cp".to_owned()),
            Self::FirstLength => ErrorFacts::Io(io::ErrorKind::PermissionDenied),
            Self::Grow => ErrorFacts::HostCommand("resize2fs".to_owned()),
            Self::FinalLength => ErrorFacts::Io(io::ErrorKind::StorageFull),
        }
    }

    fn error(self) -> Error {
        match self.expected_error() {
            ErrorFacts::HostCommand(program) => Error::HostCommand { program },
            ErrorFacts::Io(kind) => Err::<(), _>(io::Error::from(kind))
                .context("injected root I/O failure")
                .unwrap_err(),
            ErrorFacts::Unexpected => unreachable!("a fault stage has a typed error"),
        }
    }

    fn trace_length(self) -> usize {
        match self {
            Self::ImageLength => 1,
            Self::Reflink => 2,
            Self::FirstLength => 3,
            Self::Grow => 4,
            Self::FinalLength => 5,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum ErrorFacts {
    Io(io::ErrorKind),
    HostCommand(String),
    Unexpected,
}

pub(super) fn error_facts(error: Error) -> ErrorFacts {
    match error {
        Error::HostCommand { program } => ErrorFacts::HostCommand(program),
        Error::Internal(error) => error
            .source_ref()
            .downcast_ref::<io::Error>()
            .map(|source| ErrorFacts::Io(source.kind()))
            .unwrap_or(ErrorFacts::Unexpected),
        _ => ErrorFacts::Unexpected,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Operation {
    Length(PathBuf),
    Command(String, Vec<String>, Vec<u8>),
    Extend(PathBuf, u64),
}

/// Reuse one fixture across the table so verification observes every mock method.
#[derive(Clone)]
pub(super) struct RootFailure {
    pub(super) root: Arc<ReflinkRoot>,
    trace: Arc<Mutex<Vec<Operation>>>,
    stage: Arc<Mutex<Stage>>,
    extend_count: Arc<AtomicUsize>,
}

impl RootFailure {
    pub(super) fn new() -> Self {
        let trace = Arc::new(Mutex::new(Vec::new()));
        let stage = Arc::new(Mutex::new(Stage::ImageLength));
        let extend_count = Arc::new(AtomicUsize::new(0));
        let lengths = trace.clone();
        let extends = trace.clone();
        let commands = trace.clone();
        let length_stage = stage.clone();
        let extend_stage = stage.clone();
        let command_stage = stage.clone();
        let count = extend_count.clone();
        let mock = Arc::new(Unimock::new((
            RootFilesMock::length
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, path| {
                    let stage = *length_stage.lock().unwrap();
                    lengths
                        .lock()
                        .unwrap()
                        .push(Operation::Length(path.to_owned()));
                    if stage == Stage::ImageLength {
                        Err(stage.error())
                    } else {
                        Ok(80_000_000_000)
                    }
                }))
                .at_least_times(0),
            RootFilesMock::extend
                .each_call(matching!(_, _))
                .answers_arc(Arc::new(move |_, path, bytes| {
                    let stage = *extend_stage.lock().unwrap();
                    extends
                        .lock()
                        .unwrap()
                        .push(Operation::Extend(path.to_owned(), bytes));
                    let current = if count.fetch_add(1, Ordering::SeqCst) == 0 {
                        Stage::FirstLength
                    } else {
                        Stage::FinalLength
                    };
                    if stage == current {
                        Err(stage.error())
                    } else {
                        Ok(())
                    }
                }))
                .at_least_times(0),
            CommandsMock::run
                .each_call(matching!(_, _, _))
                .answers_arc(Arc::new(move |_, program, args, input| {
                    let stage = *command_stage.lock().unwrap();
                    commands.lock().unwrap().push(Operation::Command(
                        program.to_owned(),
                        args.to_vec(),
                        input.to_vec(),
                    ));
                    if (stage == Stage::Reflink && program == "cp")
                        || (stage == Stage::Grow && program == "resize2fs")
                    {
                        Err(stage.error())
                    } else {
                        Ok(Vec::new())
                    }
                }))
                .at_least_times(0),
        )));
        Self {
            root: Arc::new(ReflinkRoot {
                files: mock.clone(),
                commands: mock,
            }),
            trace,
            stage,
            extend_count,
        }
    }

    /// Select the fault and clear the trace and extension count for this row.
    pub(super) fn reset(&self, stage: Stage) {
        *self.stage.lock().unwrap() = stage;
        self.extend_count.store(0, Ordering::SeqCst);
        self.trace.lock().unwrap().clear();
    }

    /// Only this test fixture assumes that all mocked I/O is immediately ready.
    pub(super) fn prepare_ready(&self, image: &Path, copy: &Path, bytes: u64) -> Result<()> {
        self.root
            .prepare(image, copy, bytes)
            .now_or_never()
            .expect("this fixture's mocked I/O completes on the first poll")
    }

    pub(super) fn trace(&self) -> Vec<Operation> {
        self.trace.lock().unwrap().clone()
    }
}

pub(super) fn expected_trace(stage: Stage, image: &Path, copy: &Path) -> Vec<Operation> {
    let mut trace = vec![
        Operation::Length(image.to_owned()),
        Operation::Command(
            "cp".to_owned(),
            vec![
                "--reflink=always".to_owned(),
                "--sparse=auto".to_owned(),
                "--".to_owned(),
                image.to_string_lossy().into_owned(),
                copy.to_string_lossy().into_owned(),
            ],
            Vec::new(),
        ),
        Operation::Extend(copy.to_owned(), 100_000_000_000),
        Operation::Command(
            "resize2fs".to_owned(),
            vec![copy.to_string_lossy().into_owned()],
            Vec::new(),
        ),
        Operation::Extend(copy.to_owned(), 100_000_000_000),
    ];
    trace.truncate(stage.trace_length());
    trace
}

pub(super) fn layout() -> Layout {
    Layout {
        data: PathBuf::from("/fixture"),
        firecracker: PathBuf::from("/tools/firecracker"),
        jailer: PathBuf::from("/tools/jailer"),
    }
}
