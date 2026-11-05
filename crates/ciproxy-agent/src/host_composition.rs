//! Linux host preparation dependencies constructed at the CLI composition root.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tokio::sync::Mutex;

use crate::area::{AreaFiles, PersistentArea};
use crate::area_mounts::LinuxAreaMounts;
use crate::error::{ErrorResultExt, Result};
use crate::host_packages::DistributionPackages;
use crate::host_preparation::{HostPreparation, Preparation};
use crate::host_release::PINNED;
use crate::host_system::HostSystem;
use crate::host_tool_files::LinuxToolFiles;
use crate::host_tools::{HttpsHostDownload, Tools, VerifiedTools};

pub(crate) struct HostServices {
    pub(crate) preparation: Arc<dyn Preparation>,
    pub(crate) tools: Arc<dyn Tools>,
}

pub(crate) fn services(
    files: Arc<dyn AreaFiles>,
    system: Arc<dyn HostSystem>,
) -> Result<HostServices> {
    let tools: Arc<dyn Tools> = Arc::new(VerifiedTools {
        files: Arc::new(LinuxToolFiles),
        release: PINNED,
        operation: Mutex::new(()),
        download: Arc::new(HttpsHostDownload {
            client: Client::builder()
                .https_only(true)
                .connect_timeout(Duration::from_secs(20))
                .timeout(Duration::from_secs(300))
                .build()
                .context("configure pinned host downloads")?,
        }),
    });
    let preparation: Arc<dyn Preparation> = Arc::new(HostPreparation {
        files: files.clone(),
        packages: Arc::new(DistributionPackages {
            system: system.clone(),
        }),
        tools: tools.clone(),
        area: Arc::new(PersistentArea {
            files,
            mounts: Arc::new(LinuxAreaMounts { system }),
        }),
    });
    Ok(HostServices { preparation, tools })
}
