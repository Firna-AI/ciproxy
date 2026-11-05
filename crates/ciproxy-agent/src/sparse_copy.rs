//! Image zero blocks remain holes. Logical bytes and the final length stay exact.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

#[cfg_attr(test, unimock::unimock(api = SparseOutputMock))]
pub(crate) trait SparseOutput {
    fn data(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn hole(&mut self, bytes: u32) -> io::Result<()>;
    fn finish(&mut self, length: u64) -> io::Result<()>;
}

pub(crate) trait ImageCopy {
    fn copy(&self, reader: &mut dyn Read, output: &mut dyn SparseOutput) -> io::Result<u64>;
}

pub(crate) struct SparseCopy;

impl ImageCopy for SparseCopy {
    fn copy(&self, reader: &mut dyn Read, output: &mut dyn SparseOutput) -> io::Result<u64> {
        let mut buffer = [0; 64 * 1024];
        let mut length = 0;
        loop {
            let count = match reader.read(&mut buffer) {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                result => result?,
            };
            if count == 0 {
                break;
            }
            let mut offset = 0;
            while offset < count {
                let zero = buffer[offset..(offset + 4096).min(count)]
                    .iter()
                    .all(|byte| *byte == 0);
                let start = offset;
                offset = (offset + 4096).min(count);
                while offset < count
                    && buffer[offset..(offset + 4096).min(count)]
                        .iter()
                        .all(|byte| *byte == 0)
                        == zero
                {
                    offset = (offset + 4096).min(count);
                }
                if zero {
                    output.hole((offset - start) as u32)?;
                } else {
                    output.data(&buffer[start..offset])?;
                }
            }
            length += count as u64;
        }
        output.finish(length)?;
        Ok(length)
    }
}

impl SparseOutput for File {
    fn data(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.write_all(bytes)
    }
    fn hole(&mut self, bytes: u32) -> io::Result<()> {
        self.seek(SeekFrom::Current(i64::from(bytes)))?;
        Ok(())
    }
    fn finish(&mut self, length: u64) -> io::Result<()> {
        self.set_len(length)
    }
}

#[cfg(test)]
#[path = "_tests_/sparse_copy_tests.rs"]
mod sparse_copy_tests;
