//! Exact preallocation follows a reservation. Sparse holes consume no payload budget.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::Arc;

use rustix::fs::{FallocateFlags, fallocate};

use crate::error::{Error, ErrorResultExt, Result};
use crate::image_space::ImageSpace;
use crate::sparse_copy::SparseOutput;
use crate::storage_sizing::BLOCK;

pub(crate) struct BudgetFile {
    file: Box<dyn BlockWriter>,
    space: Arc<dyn ImageSpace>,
    position: u64,
}

#[cfg_attr(test, unimock::unimock(api = BlockWriterMock))]
pub(crate) trait BlockWriter: Send {
    fn allocate(&mut self, start: u64, length: u64) -> io::Result<()>;
    fn write(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn seek(&mut self, position: u64) -> io::Result<()>;
    fn length(&mut self, length: u64) -> io::Result<()>;
    fn sync(&self) -> io::Result<()>;
}

impl BlockWriter for File {
    fn allocate(&mut self, start: u64, length: u64) -> io::Result<()> {
        Ok(fallocate(&*self, FallocateFlags::KEEP_SIZE, start, length)?)
    }
    fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.write_all(bytes)
    }
    fn seek(&mut self, position: u64) -> io::Result<()> {
        Seek::seek(self, SeekFrom::Start(position))?;
        Ok(())
    }
    fn length(&mut self, length: u64) -> io::Result<()> {
        self.set_len(length)
    }
    fn sync(&self) -> io::Result<()> {
        self.sync_all()
    }
}

impl BudgetFile {
    pub(crate) fn create(path: &Path, space: Arc<dyn ImageSpace>) -> Result<Self> {
        let _claim = space.claim(32 * 1024)?;
        let file = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(path)
            .context("create bounded image file")?;
        Ok(Self::with_writer(Box::new(file), space))
    }

    pub(crate) fn with_writer(file: Box<dyn BlockWriter>, space: Arc<dyn ImageSpace>) -> Self {
        Self {
            file,
            space,
            position: 0,
        }
    }

    pub(crate) fn write(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        let start = self.position / BLOCK * BLOCK;
        let end = self
            .position
            .checked_add(bytes.len() as u64)
            .and_then(|value| value.checked_add(BLOCK - 1))
            .ok_or(Error::ImageBudget)?
            / BLOCK
            * BLOCK;
        let _claim = self.space.claim(end - start)?;
        self.file
            .allocate(start, end - start)
            .context("preallocate reserved image blocks")?;
        self.file
            .write(bytes)
            .context("write reserved image blocks")?;
        self.position += bytes.len() as u64;
        Ok(())
    }

    pub(crate) fn sync(&self) -> Result<()> {
        self.file.sync().context("sync bounded image file")
    }

    pub(crate) fn copy(&mut self, source: &Path) -> Result<()> {
        let mut input = File::open(source).context("read verified kernel")?;
        let mut bytes = [0; 64 * 1024];
        loop {
            let count = input.read(&mut bytes).context("read kernel blocks")?;
            if count == 0 {
                break;
            }
            self.write(&bytes[..count])?;
        }
        self.sync()
    }
}

impl SparseOutput for BudgetFile {
    fn data(&mut self, bytes: &[u8]) -> io::Result<()> {
        match self.write(bytes) {
            Ok(()) => Ok(()),
            Err(error) => Err(io::Error::other(error)),
        }
    }
    fn hole(&mut self, bytes: u32) -> io::Result<()> {
        self.position += u64::from(bytes);
        self.file.seek(self.position)?;
        Ok(())
    }
    fn finish(&mut self, length: u64) -> io::Result<()> {
        self.file.length(length)
    }
}

#[cfg(test)]
#[path = "_tests_/budget_file_tests.rs"]
mod budget_file_tests;
