//! `import-image`: a test and operator path that publishes a local
//! `macos.tar` with no image host.
//!
//! Run it as root after `prepare-host` and before `install`, so the daemon
//! loads the image when it starts. It takes the runtime lock, so it refuses
//! to run while the agent daemon runs.

use std::path::PathBuf;
use std::sync::Arc;

use super::{MacPlatform, area};
use crate::boundaries::Images;
use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::image_bundle::folder_name;
use crate::macos::image_import::{ImageImport, copy_checked};
use crate::macos::paths::CONTROL;
use crate::platform::Platform;
use crate::runtime_ownership::{CliRuntimeWaitNotice, FileRuntimeClaims, RuntimeClaims};

pub(crate) async fn import_image(request: ImageImport) -> Result<()> {
    let platform = MacPlatform::new();
    platform.host().require_root()?;
    let record = area::record()?.ok_or(Error::Preparation {
        component: "area volume",
        repair: "Run prepare-host first to create the area volume. Then rerun import-image.",
    })?;
    area::mount(&record)?;
    let claims = FileRuntimeClaims {
        control: PathBuf::from(CONTROL),
        notice: Arc::new(CliRuntimeWaitNotice),
    };
    let _owner = claims.claim().map_err(|error| {
        tracing::warn!(%error, "the runtime lock is not free");
        Error::Preparation {
            component: "runtime lock",
            repair: "Stop the agent daemon with `launchctl bootout system/dev.ciproxy.agent`. Then rerun import-image.",
        }
    })?;
    let images = platform.images.clone();
    images.recover()?;
    if images.known(&request.image_id) {
        println!("image {} is already published", request.image_id.0);
        return Ok(());
    }
    println!("copying and checking the archive; a large archive takes some minutes");
    let archive = images.archive(&request.image_id);
    let (source, target, sha256) = (
        request.archive.clone(),
        archive.clone(),
        request.sha256.clone(),
    );
    tokio::task::spawn_blocking(move || copy_checked(&source, &target, &sha256))
        .await
        .context("join the archive check")??;
    println!("archive checked: size and SHA-256 match");
    if let Err(error) = images
        .publish(&request.image_id, request.root_disk_bytes, &archive)
        .await
    {
        // Recovery removes the archive and the pending folder too, but a
        // failed import should not leave them on the volume until then.
        if let Err(cleanup) = images.recover() {
            tracing::warn!(error = %cleanup, "the daemon start removes the partial import");
        }
        return Err(error);
    }
    println!(
        "image {} published as images/{} and made the newest image",
        request.image_id.0,
        folder_name(&request.image_id)
    );
    Ok(())
}
