# SPDX-License-Identifier: Apache-2.0
packer {
  required_plugins {
    qemu = {
      version = "= 1.1.8"
      source  = "github.com/hashicorp/qemu"
    }
  }
}

variable "work_dir" { type = string }
variable "guest_binary" { type = string }
variable "image_definition" { type = string }
variable "build_cpus" { default = 4 }
variable "build_memory_mb" { default = 16384 }

source "qemu" "image" {
  accelerator            = "kvm"
  disk_image             = true
  iso_url                = "${var.work_dir}/ubuntu.qcow2"
  iso_checksum           = "file:${var.work_dir}/ubuntu.sha256"
  output_directory       = "${var.work_dir}/qemu-output"
  vm_name                = "ciproxy.qcow2"
  format                 = "qcow2"
  disk_size              = "80G"
  disk_interface         = "virtio"
  net_device             = "virtio-net"
  cpus                   = var.build_cpus
  memory                 = var.build_memory_mb
  headless               = true
  cd_files               = ["${var.work_dir}/seed/user-data", "${var.work_dir}/seed/meta-data"]
  cd_label               = "cidata"
  ssh_username           = "packer"
  ssh_private_key_file   = "${var.work_dir}/build-key"
  ssh_timeout            = "30m"
  shutdown_command       = "sudo /sbin/poweroff"
  shutdown_timeout       = "10m"
}
