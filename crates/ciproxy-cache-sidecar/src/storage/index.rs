//! Recoverable metadata reads and deterministic matching of local entries.

use std::collections::HashSet;
use std::path::Path;

use tokio::fs;

use crate::error::{CacheError, Result, ResultExt};

use super::interface::{Entry, EntryOrigin, Lookup};

pub(super) async fn entries(root: &Path) -> Result<Vec<Entry>> {
    fs::create_dir_all(root).await.internal()?;
    let mut files = fs::read_dir(root).await.internal()?;
    let mut entries = Vec::new();
    while let Some(file) = files.next_entry().await.internal()? {
        let path = file.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let bytes = fs::read(&path).await.internal()?;
            match serde_json::from_slice::<Entry>(&bytes) {
                Ok(entry) if uuid::Uuid::parse_str(&entry.id).is_ok() => entries.push(entry),
                _ => tracing::warn!("Ignoring corrupt local cache metadata"),
            }
        }
    }
    Ok(entries)
}

pub(super) fn matching(entries: &[Entry], lookup: &Lookup) -> Option<Entry> {
    for scope in &lookup.scopes {
        for key in &lookup.keys {
            let candidates: Vec<_> = entries
                .iter()
                .filter(|entry| entry.scope == *scope && entry.version == lookup.version)
                .collect();
            let saved_keys: HashSet<_> = candidates
                .iter()
                .filter(|entry| entry.origin == EntryOrigin::AcceptedSave)
                .map(|entry| entry.key.as_str())
                .collect();
            let candidates: Vec<_> = candidates
                .iter()
                .copied()
                .filter(|entry| {
                    entry.origin == EntryOrigin::AcceptedSave
                        || !saved_keys.contains(entry.key.as_str())
                })
                .collect();
            if let Some(exact) = candidates.iter().find(|entry| entry.key == *key) {
                return Some((*exact).clone());
            }
            if let Some(prefix) = candidates
                .into_iter()
                .filter(|entry| entry.key.starts_with(key))
                .max_by_key(|entry| (entry.created, &entry.id))
            {
                return Some(prefix.clone());
            }
        }
    }
    None
}

pub(super) async fn remove(root: &Path, entry: &Entry) -> Result<()> {
    let mut names = blob_names(entry);
    names.push(format!("{}.json", entry.id));
    for name in names {
        let path = root.join(name);
        match fs::remove_file(path).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            result => result.internal()?,
        }
    }
    Ok(())
}

pub(super) fn blob_names(entry: &Entry) -> Vec<String> {
    if entry.parts.is_empty() {
        vec![format!("{}.blob", entry.id)]
    } else {
        entry
            .parts
            .iter()
            .enumerate()
            .map(|(index, _)| format!("{}.{index}.blob", entry.id))
            .collect()
    }
}

pub(super) fn next_order(values: impl Iterator<Item = u64>) -> Result<u64> {
    values
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or(CacheError::Unavailable)
}
