#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Keep every full-image provisioner and replace only Azure packaging."""

import argparse
import pathlib
import shutil


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("upstream", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    args = parser.parse_args()
    templates = args.upstream.resolve() / "images/ubuntu/templates"
    recipe = (templates / "build.ubuntu-24_04.pkr.hcl").read_text()
    recipe = recipe.replace('sources = ["source.azure-arm.image"]',
                            'sources = ["source.qemu.image"]')
    recipe = recipe.replace("${path.root}", str(templates))
    azure_cleanup = '''  provisioner "shell" {
    execute_command = "sudo sh -c '{{ .Vars }} {{ .Path }}'"
    inline          = ["sleep 30", "/usr/sbin/waagent -force -deprovision+user && export HISTSIZE=0 && sync"]
  }
'''
    if azure_cleanup not in recipe:
        raise RuntimeError("Pinned upstream Azure packaging block changed")
    recipe = recipe.replace(azure_cleanup, "")
    # Upstream "Apt sources resolve through the mirror list" rewrites Azure's URI.
    # Cloud-init drops the generic archive URI's trailing slash.
    # Normalize the generic base before that provisioner owns the mirror list.
    bootstrap = '''
  provisioner "shell" {
    inline = [
      "sudo cloud-init status --wait",
      "sudo apt-get update",
      "sudo apt-get install -y walinuxagent grub2-common",
      "sudo sed -i -E 's|http://archive[.]ubuntu[.]com/ubuntu/?$|http://azure.archive.ubuntu.com/ubuntu/|' /etc/apt/sources.list.d/ubuntu.sources"
    ]
  }
'''
    marker = '  name = "ubuntu-24_04"'
    recipe = recipe.replace(marker, marker + "\n" + bootstrap)
    packaging = '''
  provisioner "file" {
    source = var.guest_binary
    destination = "/tmp/ciproxy-guest"
  }
  provisioner "shell" {
    inline = ["mkdir -p /tmp/ciproxy-image"]
  }
  provisioner "file" {
    source = var.cache_binary
    destination = "/tmp/ciproxy-cache-sidecar"
  }
  provisioner "file" {
    source = var.worker_binary
    destination = "/tmp/ciproxy-worker.dll"
  }
  provisioner "file" {
    source = "${var.image_definition}/"
    destination = "/tmp/ciproxy-image"
  }
  provisioner "file" {
    source = "${var.work_dir}/runner.tar.gz"
    destination = "/tmp/ciproxy-runner.tar.gz"
  }
  provisioner "file" {
    source = "${var.work_dir}/vmlinux"
    destination = "/tmp/vmlinux-ciproxy"
  }
  provisioner "shell" {
    inline = ["sudo bash /tmp/ciproxy-image/provision.sh"]
  }
'''
    recipe = recipe.rstrip().removesuffix("}") + packaging + "}\n"
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / "build.pkr.hcl").write_text(recipe)
    shutil.copy(templates / "variable.ubuntu.pkr.hcl", args.output)
    shutil.copy(pathlib.Path(__file__).with_name("source.qemu.pkr.hcl"), args.output)


if __name__ == "__main__":
    main()
