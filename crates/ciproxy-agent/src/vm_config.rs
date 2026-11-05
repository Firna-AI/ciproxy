//! Firecracker boot configuration contains no token, registration, or job facts.

use serde::Serialize;

use crate::state::VmRecord;

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct VmConfig {
    boot_source: BootSource,
    drives: [Drive; 2],
    machine_config: MachineConfig,
    network_interfaces: [NetworkInterface; 1],
    vsock: Vsock,
}

#[derive(Serialize)]
struct BootSource {
    kernel_image_path: &'static str,
    boot_args: &'static str,
}

#[derive(Serialize)]
struct Drive {
    drive_id: &'static str,
    path_on_host: &'static str,
    is_root_device: bool,
    is_read_only: bool,
}

#[derive(Serialize)]
struct MachineConfig {
    vcpu_count: u32,
    mem_size_mib: u64,
}

#[derive(Serialize)]
struct NetworkInterface {
    iface_id: &'static str,
    host_dev_name: &'static str,
    guest_mac: &'static str,
}

#[derive(Serialize)]
struct Vsock {
    guest_cid: u32,
    uds_path: &'static str,
}

impl VmConfig {
    pub(crate) fn for_vm(vm: &VmRecord) -> Self {
        Self {
            boot_source: BootSource {
                kernel_image_path: "/vmlinux",
                boot_args: "console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw ip=172.30.0.2::172.30.0.1:255.255.255.252::eth0:off",
            },
            drives: [
                Drive {
                    drive_id: "rootfs",
                    path_on_host: "/rootfs.ext4",
                    is_root_device: true,
                    is_read_only: false,
                },
                Drive {
                    drive_id: "docker",
                    path_on_host: "/docker.ext4",
                    is_root_device: false,
                    is_read_only: false,
                },
            ],
            machine_config: MachineConfig {
                vcpu_count: vm.report.size.vcpu,
                mem_size_mib: vm.report.size.memory_bytes.div_ceil(1_048_576),
            },
            network_interfaces: [NetworkInterface {
                iface_id: "eth0",
                host_dev_name: "tap0",
                guest_mac: "06:00:AC:1E:00:02",
            }],
            vsock: Vsock {
                guest_cid: u32::from(vm.network_index) + 3,
                uds_path: "/vsock.sock",
            },
        }
    }
}
