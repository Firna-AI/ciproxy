//! Bounded tee capture that never stops a GitHub transfer on a disk fault.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::Body;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use tempfile::{NamedTempFile, TempPath};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

struct Charge {
    valid: bool,
    bytes: u64,
}

pub(crate) struct Reservation {
    pub path: PathBuf,
    charge: Mutex<Charge>,
    used: Arc<AtomicU64>,
    capacity: Arc<AtomicU64>,
}

impl Reservation {
    pub(super) fn new(path: PathBuf, used: Arc<AtomicU64>, capacity: Arc<AtomicU64>) -> Self {
        Self {
            path,
            charge: Mutex::new(Charge {
                valid: true,
                bytes: 0,
            }),
            used,
            capacity,
        }
    }
    fn claim(&self, length: u64) -> bool {
        let Ok(mut charge) = self.charge.lock() else {
            return false;
        };
        if !charge.valid {
            return false;
        }
        if self
            .used
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |used| {
                used.checked_add(length)
                    .filter(|next| *next <= self.capacity.load(Ordering::SeqCst))
            })
            .is_err()
        {
            return false;
        }
        charge.bytes += length;
        true
    }

    pub fn release(&self, published: bool) {
        if let Ok(mut charge) = self.charge.lock()
            && charge.valid
        {
            charge.valid = false;
            if !published {
                self.used.fetch_sub(charge.bytes, Ordering::SeqCst);
            }
            charge.bytes = 0;
        }
    }
}

impl Drop for Reservation {
    fn drop(&mut self) {
        self.release(false);
    }
}

/// A complete transient archive. Its disk space is released on drop.
pub struct Captured {
    pub(crate) path: TempPath,
    pub(crate) size: u64,
    pub(crate) digest: String,
    pub(crate) generation: u64,
    pub(crate) reservation: Arc<Reservation>,
    pub(crate) parts: Vec<Arc<Captured>>,
}

impl Drop for Captured {
    fn drop(&mut self) {
        self.reservation.release(false);
    }
}

/// Completion handle of an observed HTTP stream.
pub struct Capture {
    /// A complete capture, or none after a disk fault or partial transfer.
    pub complete: tokio::sync::Mutex<Option<Arc<Captured>>>,
    pub(crate) reservation: Arc<Reservation>,
}

pub(crate) fn tee(
    temporary: NamedTempFile,
    body: Body,
    used: Arc<AtomicU64>,
    capacity: Arc<AtomicU64>,
    generation: u64,
    expected: Option<u64>,
) -> (Body, Arc<Capture>) {
    let path = temporary.into_temp_path();
    let reservation = Arc::new(Reservation {
        path: path.to_path_buf(),
        charge: Mutex::new(Charge {
            valid: true,
            bytes: 0,
        }),
        used,
        capacity,
    });
    let capture = Arc::new(Capture {
        complete: tokio::sync::Mutex::new(None),
        reservation: reservation.clone(),
    });
    let result = capture.clone();
    let stream = async_stream::stream! {
        let mut blob = Some(Captured { path, size: 0, digest: String::new(), generation, reservation, parts: Vec::new() });
        let mut digest = Sha256::new();
        let mut source = body.into_data_stream();
        while let Some(chunk) = source.next().await {
            let chunk = match chunk { Ok(chunk) => chunk, Err(error) => { yield Err(error); return; } };
            if let Some(active) = &mut blob {
                let length = chunk.len() as u64;
                let written = if active.reservation.claim(length) {
                    match OpenOptions::new().append(true).open(&active.path).await {
                        Ok(mut writer) => writer.write_all(&chunk).await.is_ok(),
                        Err(_) => false,
                    }
                } else { false };
                if written { active.size += length; digest.update(&chunk); }
                else { blob = None; }
            }
            if blob.as_ref().is_some_and(|active| expected == Some(active.size))
                && let Some(mut complete) = blob.take() {
                    complete.digest = hex(&digest.clone().finalize());
                    *result.complete.lock().await = Some(Arc::new(complete));
                }
            yield Ok::<_, axum::Error>(chunk);
        }
        if let Some(mut complete) = blob
            && expected.is_none_or(|size| complete.size == size) {
                complete.digest = hex(&digest.finalize());
                *result.complete.lock().await = Some(Arc::new(complete));
            }
    };
    (Body::from_stream(stream), capture)
}

pub(super) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
