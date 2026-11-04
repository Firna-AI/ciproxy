//! Short-lived file handles for archives composed of one or more blobs.

use std::path::Path;

use bytes::Bytes;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::error::{Result, ResultExt};

use super::index::blob_names;
use super::interface::ReadSource;

pub(super) async fn read(
    root: &Path,
    source: &ReadSource,
    offset: u64,
    length: usize,
) -> Result<Bytes> {
    let (path, offset, available) = match source {
        ReadSource::Entry(entry) if entry.parts.is_empty() => (
            root.join(format!("{}.blob", entry.id)),
            offset,
            entry.size.saturating_sub(offset),
        ),
        ReadSource::Capture(blob) if blob.parts.is_empty() => (
            blob.path.to_path_buf(),
            offset,
            blob.size.saturating_sub(offset),
        ),
        ReadSource::Entry(entry) => {
            let Some((index, offset, available)) = segment(&entry.parts, offset) else {
                return Ok(Bytes::new());
            };
            (root.join(&blob_names(entry)[index]), offset, available)
        }
        ReadSource::Capture(blob) => {
            let lengths: Vec<_> = blob.parts.iter().map(|part| part.size).collect();
            let Some((index, offset, available)) = segment(&lengths, offset) else {
                return Ok(Bytes::new());
            };
            (blob.parts[index].path.to_path_buf(), offset, available)
        }
    };
    let mut file = File::open(path).await.internal()?;
    file.seek(std::io::SeekFrom::Start(offset))
        .await
        .internal()?;
    let mut bytes = vec![0; length.min(65536).min(available as usize)];
    let count = file.read(&mut bytes).await.internal()?;
    bytes.truncate(count);
    Ok(Bytes::from(bytes))
}

fn segment(lengths: &[u64], mut offset: u64) -> Option<(usize, u64, u64)> {
    for (index, length) in lengths.iter().enumerate() {
        if offset < *length {
            return Some((index, offset, length - offset));
        }
        offset = offset.saturating_sub(*length);
    }
    None
}
