//! Track the Worker log pages of one runner.

mod steps;

use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, ErrorKind, Read, Seek, SeekFrom};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use ciproxy_protocol::usage::JobStep;

use self::steps::Steps;

const DIAG: &str = "/opt/actions-runner/_diag";
const MAX_DIRECTORY_ENTRIES: usize = 4_096;
const MAX_LINE_BYTES: u64 = 1024 * 1024;
const MAX_REPORT_READ_BYTES: u64 = 64 * 1024 * 1024;

/// List and read the runner's log files without exposing their contents in errors.
#[cfg_attr(test, unimock::unimock(api = LogPagesMock))]
pub(crate) trait LogPages: Send {
    /// List Worker page names.
    fn list(&self) -> io::Result<Vec<String>>;
    /// Open a page at the byte after the last complete line read.
    fn open(&self, name: &str, offset: u64) -> io::Result<Box<dyn BufRead>>;
}

/// The runner log directory in both the Linux and macOS images.
pub(crate) struct DiagDirectory;

impl LogPages for DiagDirectory {
    fn list(&self) -> io::Result<Vec<String>> {
        let entries = match fs::read_dir(DIAG) {
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            entries => entries?,
        };
        let mut pages = Vec::new();
        for (index, entry) in entries.enumerate() {
            if index == MAX_DIRECTORY_ENTRIES {
                return Err(ErrorKind::InvalidData.into());
            }
            if let Ok(name) = entry?.file_name().into_string()
                && is_worker_page(&name)
            {
                pages.push(name);
            }
        }
        Ok(pages)
    }

    fn open(&self, name: &str, offset: u64) -> io::Result<Box<dyn BufRead>> {
        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(Path::new(DIAG).join(name))?;
        let metadata = file.metadata()?;
        if !metadata.is_file() || metadata.len() < offset {
            return Err(ErrorKind::InvalidData.into());
        }
        file.seek(SeekFrom::Start(offset))?;
        Ok(Box::new(BufReader::new(file)))
    }
}

fn is_worker_page(name: &str) -> bool {
    name.strip_prefix("Worker_")
        .and_then(|stamp| stamp.strip_suffix("-utc.log"))
        .and_then(|stamp| stamp.split_once('-'))
        .is_some_and(|(date, time)| {
            date.len() == 8
                && time.len() == 6
                && date
                    .bytes()
                    .chain(time.bytes())
                    .all(|byte| byte.is_ascii_digit())
        })
}

struct Cursor {
    page: String,
    offset: u64,
}

/// The pages and observations of the only Worker started by this runner.
pub(super) struct WorkerLog {
    pages: Box<dyn LogPages>,
    excluded: Vec<String>,
    cursor: Option<Cursor>,
    steps: Steps,
}

impl WorkerLog {
    /// Create a reader for the runner's log directory.
    pub(super) fn new(pages: Box<dyn LogPages>) -> Self {
        Self {
            pages,
            excluded: Vec::new(),
            cursor: None,
            steps: Steps::default(),
        }
    }

    /// Ignore pages that already exist before the listener starts.
    pub(super) fn exclude_existing_pages(&mut self) -> io::Result<()> {
        self.excluded = self.pages.list()?;
        self.excluded.sort_unstable();
        Ok(())
    }

    /// Read all new complete lines and return the known job steps.
    pub(super) fn report(&mut self) -> io::Result<Vec<JobStep>> {
        if !self.steps.full() {
            self.follow()?;
        }
        Ok(self.steps.snapshot())
    }

    fn follow(&mut self) -> io::Result<()> {
        let mut pages = self.pages.list()?;
        pages.retain(|page| self.excluded.binary_search(page).is_err());
        pages.sort_unstable();
        let (first, mut offset) = match &self.cursor {
            None => (0, 0),
            Some(cursor) => {
                let Ok(index) = pages.binary_search(&cursor.page) else {
                    return Err(ErrorKind::NotFound.into());
                };
                (index, cursor.offset)
            }
        };
        let mut budget = MAX_REPORT_READ_BYTES;
        for page in pages.into_iter().skip(first) {
            let reader = self.pages.open(&page, offset)?;
            let next_offset = self.read_page(reader, offset, &mut budget)?;
            self.cursor = Some(Cursor {
                page,
                offset: next_offset,
            });
            if self.steps.full() {
                break;
            }
            offset = 0;
        }
        Ok(())
    }

    fn read_page(
        &mut self,
        mut reader: Box<dyn BufRead>,
        mut offset: u64,
        budget: &mut u64,
    ) -> io::Result<u64> {
        let mut line = Vec::new();
        while !self.steps.full() {
            line.clear();
            let count = (&mut reader)
                .take(MAX_LINE_BYTES + 1)
                .read_until(b'\n', &mut line)? as u64;
            *budget = budget.checked_sub(count).ok_or(ErrorKind::InvalidData)?;
            if count > MAX_LINE_BYTES {
                return Err(ErrorKind::InvalidData.into());
            }
            if !line.ends_with(b"\n") {
                break;
            }
            let Ok(text) = str::from_utf8(&line) else {
                return Err(ErrorKind::InvalidData.into());
            };
            self.steps.observe(text.trim_end_matches(['\r', '\n']))?;
            offset += count;
        }
        Ok(offset)
    }
}

#[cfg(test)]
#[path = "_tests_/follow_tests.rs"]
mod follow_tests;
