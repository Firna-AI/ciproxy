//! Persist source enrichment without changing archive identity or access order.

use std::path::Path;

use ciproxy_protocol::cache_metadata::CacheEntryMetadata;
use tokio::fs;

use crate::error::{CacheError, Result, ResultExt};

use super::interface::{Entry, EntryOrigin};

pub(super) async fn annotate(
    root: &Path,
    entry: &Entry,
    source: &CacheEntryMetadata,
) -> Result<()> {
    let path = root.join(format!("{}.json", entry.id));
    let mut current: Entry =
        serde_json::from_slice(&fs::read(&path).await.internal()?).internal()?;
    if current.id != entry.id
        || current.scope != entry.scope
        || current.key != source.key
        || current.version != source.version
        || current.scope.reference != source.git_ref.0
        || current.origin != EntryOrigin::AcceptedSave
        || current.upstream_id.as_ref() != Some(&source.id)
    {
        return Err(CacheError::Unavailable);
    }
    if current.source.is_some() {
        return Ok(());
    }
    current.source = Some(source.clone());
    let temporary = root.join(format!("{}.pending", entry.id));
    fs::write(&temporary, serde_json::to_vec(&current).internal()?)
        .await
        .internal()?;
    fs::rename(temporary, path).await.internal()
}
