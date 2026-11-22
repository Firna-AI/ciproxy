//! Linux parts: systemd, Firecracker and its jailer, XFS, ext4, and nftables.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};

use crate::application_cache;
use crate::area::AreaFiles;
use crate::area_files::LinuxAreaFiles;
use crate::boundaries::{DockerDisks, Host, Service, Updates, Vms};
use crate::docker::LinuxDockers;
use crate::error::{Error, Result};
use crate::hardware::LinuxHost;
use crate::host_composition::{HostServices, services};
use crate::host_file_ops::directory;
use crate::host_paths::{AREA, CONTROL, tools};
use crate::host_preparation::{FinalUninstall, uninstall_inventory};
use crate::host_system::LinuxHostSystem;
use crate::layout::Layout;
use crate::network::LinuxNetwork;
use crate::preparation::{Preparation, Uninstall, stage_error};
use crate::process::{Commands, LinuxCommands};
use crate::storage_sizing::{DEFAULT_IMAGE_BYTES, setting};
use crate::system_service::SystemService;
use crate::updates::AgentUpdates;
use crate::vm_files::LinuxVmFiles;
use crate::vm_root::{LinuxRootFiles, ReflinkRoot};
use crate::vms::LinuxVms;
use crate::vsock::LinuxGuests;
use crate::workload_space::WorkloadSpace;

use super::{Machine, MachineInput, Platform, UninstallParts};

/// The Linux adapters that every invocation shares.
pub(crate) struct LinuxPlatform {
    layout: Layout,
    files: Arc<dyn AreaFiles>,
    system: Arc<LinuxHostSystem>,
    commands: Arc<dyn Commands>,
}

/// Host preparation parts that the VM backend reuses.
pub(crate) struct LinuxPrepared {
    services: HostServices,
    requested_budget: Option<u64>,
}

impl LinuxPlatform {
    pub(crate) fn new() -> Self {
        Self {
            layout: Layout {
                data: PathBuf::from(AREA),
                firecracker: tools().join("firecracker"),
                jailer: tools().join("jailer"),
            },
            files: Arc::new(LinuxAreaFiles::new()),
            system: Arc::new(LinuxHostSystem),
            commands: Arc::new(LinuxCommands),
        }
    }
}

#[async_trait]
impl Platform for LinuxPlatform {
    type Prepared = LinuxPrepared;

    fn service(&self) -> Arc<dyn Service> {
        Arc::new(SystemService)
    }

    fn host(&self) -> Arc<dyn Host> {
        Arc::new(LinuxHost {
            system: self.system.clone(),
            area: self.files.clone(),
        })
    }

    fn prepare(&self) -> Result<(Arc<dyn Preparation>, LinuxPrepared)> {
        if let Err(error) = directory(Path::new(CONTROL)) {
            return Err(stage_error(
                error,
                "control directory",
                "Restore root ownership and write access to /var/lib/ciproxy. Then rerun install.",
            ));
        }
        let requested = match std::env::var("CIPROXY_IMAGE_BUDGET_BYTES") {
            Ok(value) => Some(value),
            Err(std::env::VarError::NotPresent) => None,
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err(Error::Preparation {
                    component: "image budget setting",
                    repair: "Set CIPROXY_IMAGE_BUDGET_BYTES to an integer byte count.",
                });
            }
        };
        let requested_budget = setting(requested.as_deref())?;
        let services = services(self.files.clone(), self.system.clone(), requested_budget)?;
        Ok((
            services.preparation.clone(),
            LinuxPrepared {
                services,
                requested_budget,
            },
        ))
    }

    fn machine(&self, prepared: LinuxPrepared, input: MachineInput<'_>) -> Result<Machine> {
        let inventory = if input.uninstall {
            uninstall_inventory(self.files.as_ref())?
        } else {
            self.files.inspect()?
        };
        let image_budget = inventory.record.map_or(
            prepared.requested_budget.unwrap_or(DEFAULT_IMAGE_BYTES),
            |record| record.image_budget,
        );
        let image_services = crate::application_images::prepare(
            &self.layout,
            image_budget,
            input.claims,
            input.owner,
        )?;
        let cache_services = application_cache::prepare(self.commands.clone(), input.identity)?;
        let cache = cache_services.cache.clone();
        let (guest_tx, guests) = mpsc::channel(256);
        let commands = &self.commands;
        let layout = &self.layout;
        let guest_channels = Arc::new(LinuxGuests::new(layout.clone(), commands.clone(), guest_tx));
        let vms: Arc<dyn Vms> = Arc::new(LinuxVms {
            tools: prepared.services.tools,
            layout: layout.clone(),
            commands: commands.clone(),
            network: Arc::new(LinuxNetwork::new(
                commands.clone(),
                prepared.services.network_packages,
            )),
            files: Arc::new(LinuxVmFiles {
                layout: layout.clone(),
                root: Arc::new(ReflinkRoot {
                    files: Arc::new(LinuxRootFiles),
                    commands: commands.clone(),
                }),
                space: image_services.space,
            }),
            guests: guest_channels,
            operation: Mutex::new(()),
        });
        Ok(Machine {
            image_budget,
            images: image_services.images,
            cache,
            vms,
            guests,
            tasks: Box::new(cache_services),
        })
    }

    fn dockers(
        &self,
        allocation: Arc<Mutex<()>>,
        space: Arc<dyn WorkloadSpace>,
        image_budget: u64,
    ) -> Option<Arc<dyn DockerDisks>> {
        Some(Arc::new(LinuxDockers {
            layout: self.layout.clone(),
            commands: self.commands.clone(),
            allocation,
            space,
            image_budget,
        }))
    }

    async fn install_cache(&self) -> Result<()> {
        application_cache::installer(self.commands.clone())
            .install()
            .await
    }

    fn updates(&self) -> Arc<dyn Updates> {
        Arc::new(AgentUpdates {
            commands: self.commands.clone(),
            release_server: std::env::var("CIPROXY_RELEASE_SERVER_URL")
                .unwrap_or_else(|_| "https://get.ciproxy.dev".to_owned()),
        })
    }

    fn uninstall(&self, parts: UninstallParts) -> Arc<dyn Uninstall> {
        Arc::new(FinalUninstall {
            _runtime: parts.runtime,
            files: self.files.clone(),
            images: parts.images,
            preparation: parts.preparation,
            service: parts.service,
            store: parts.store,
            agent: parts.agent,
        })
    }
}
