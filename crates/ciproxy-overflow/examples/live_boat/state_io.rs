//! Private durable metadata for the live Boat commands.

use std::{fs, io::Write, os::unix::fs::OpenOptionsExt, path::Path};

use serde::{Serialize, de::DeserializeOwned};

use crate::live_support::{Result, SafeResult};

pub(crate) fn read<T: DeserializeOwned>(path: &Path) -> Result<T> {
    serde_json::from_slice(&fs::read(path).safe("read state file")?).safe("decode state file")
}

pub(crate) fn save(path: &Path, value: &impl Serialize) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value).safe("encode state")?;
    let temporary = path.with_extension("pending");
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&temporary)
            .safe("create state file")?;
        file.write_all(&bytes).safe("write state")?;
        file.sync_all().safe("sync state")?;
    }
    fs::rename(temporary, path).safe("commit state")?;
    fs::File::open(path.parent().unwrap_or(Path::new(".")))
        .safe("open state directory")?
        .sync_all()
        .safe("sync state directory")
}

pub(crate) fn lock(path: &Path) -> Result<fs::File> {
    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path.with_extension("lock"))
        .safe("open state lock")?;
    file.lock().safe("lock live probe state")?;
    Ok(file)
}
