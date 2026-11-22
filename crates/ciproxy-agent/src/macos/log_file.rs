//! An append-only log file that follows a rotation by newsyslog.
//!
//! newsyslog renames the log and sends no signal. Every few seconds, a
//! write checks whether the path still names the open file. When the path
//! is gone or names another file, the writer opens the path again. launchd
//! keeps its own descriptor only for early output, such as a panic.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CHECK_INTERVAL: Duration = Duration::from_secs(5);

struct Open {
    file: File,
    inode: u64,
    checked: Instant,
}

/// A shared log file. Clone the `Arc` for each writer.
pub(crate) struct ReopeningLog {
    path: PathBuf,
    open: Mutex<Open>,
}

impl ReopeningLog {
    pub(crate) fn open(path: &Path) -> io::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            path: path.to_owned(),
            open: Mutex::new(open(path)?),
        }))
    }

    fn write_line(&self, bytes: &[u8]) -> io::Result<()> {
        let mut current = self
            .open
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if current.checked.elapsed() >= CHECK_INTERVAL {
            let moved =
                std::fs::metadata(&self.path).map_or(true, |meta| meta.ino() != current.inode);
            if moved {
                *current = open(&self.path)?;
            } else {
                current.checked = Instant::now();
            }
        }
        current.file.write_all(bytes)
    }
}

/// One writer of the shared log, for `tracing_subscriber`.
pub(crate) struct LogWriter(pub(crate) Arc<ReopeningLog>);

impl Write for LogWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.write_line(bytes)?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Append with mode `0600`. Each JSON line is one small append, so the
/// lines of the agent and of its VM helpers do not mix.
fn open(path: &Path) -> io::Result<Open> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(path)?;
    Ok(Open {
        inode: file.metadata()?.ino(),
        file,
        checked: Instant::now(),
    })
}
