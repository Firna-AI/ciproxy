# SPDX-License-Identifier: Apache-2.0
"""Keep source provisioning inside a chroot and export artifacts in scratch."""

import json
import os
import pathlib
import re
import shutil
import subprocess
import tempfile

from pin_recipe import PINNED_RECIPE_DIRECTORY
from _checks_.image_cleanup import legacy_image


SOURCE_FSTAB = ("UUID=source-root / ext4 discard,errors=remount-ro 0 1\n"
                "UUID=source-efi /boot/efi vfat umask=0077 0 1\n")
RELEASE_FSTAB = "/dev/vda / ext4 defaults 0 1\n"
CAPABILITY = "0100000200200000000000000000000000000000"


def provision_sandbox(source, image):
    for relative in ["usr/bin", "dev", "image", "etc/ciproxy", "etc/sudoers.d",
                     "etc/systemd/network", "etc/systemd/system", "usr/local/bin", "sbin"]:
        (source / relative).mkdir(parents=True, exist_ok=True)
    for name in ["provision.sh", "boat-build-clean-image.sh", "pins.env", "ciproxy-guest.service"]:
        shutil.copyfile(image / name, source / "image" / name)
    # The Worker install has its own checks, so the export fixture stubs it.
    (source / "image/runner-patch").mkdir()
    (source / "image/runner-patch/install-worker.sh").write_text("exit 0\n")
    for name in ["bash", "rm"]:
        binary = pathlib.Path(shutil.which(name)).resolve()
        with binary.open("rb") as command:
            header = command.readline(256)
        executable = pathlib.Path(header[2:].split()[0].decode()) if header.startswith(b"#!") else binary
        libraries = subprocess.check_output(["ldd", str(executable)], text=True)
        for path in [str(binary), str(executable), *re.findall(r"(/[^\s()]+)", libraries)]:
            destination = source / path.lstrip("/")
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, destination)
            destination.chmod(0o755)
        shutil.copyfile(binary, source / "usr/bin" / name)
        (source / "usr/bin" / name).chmod(0o755)
    (source / "dev/null").touch()
    stub = source / "usr/bin/provision-stub"
    stub.write_text('''#!/usr/bin/bash
case "${0##*/}" in
    dirname) printf '%s\\n' "${1%/*}" ;;
    cat) while IFS= read -r line; do printf '%s\\n' "$line"; done ;;
    install)
        if [[ ${@: -2:1} == /image/ciproxy-guest.service ]]; then
            while IFS= read -r line; do printf '%s\\n' "$line"; done \\
                < /image/ciproxy-guest.service > /etc/systemd/system/ciproxy-guest.service
        fi ;;
    sha256sum) while IFS= read -r line; do :; done ;;
esac
exit 0
''')
    stub.chmod(0o755)
    for name in ["reboot", "poweroff"]:
        command = source / "sbin" / name
        command.write_text('#!/usr/bin/bash\nprintf \'%s\\n\' "${0##*/}" "$@" > /reset-observation\n')
        command.chmod(0o755)
    for name in ["dirname", "cat", "sha256sum", "install", "id", "useradd",
                 "chmod", "chown", "tar", "cp", "find", "systemctl"]:
        (source / "usr/bin" / name).symlink_to(stub.name)


def create(workspace, image):
    workspace.mkdir(parents=True, exist_ok=True)
    root = pathlib.Path(tempfile.mkdtemp(prefix="export-check-", dir=workspace)).resolve()
    source = root / "source"
    work = root / "work"
    provision_sandbox(source, image)
    legacy_image(source, preferences=False)
    (source / "etc/fstab").write_text(SOURCE_FSTAB)
    (source / "var/lib/dpkg").mkdir(parents=True)
    (source / "var/lib/dpkg/status").write_text(
        "Package: temurin-17-jdk\nStatus: install ok installed\nVersion: 17.0.22.0.0+7-0\n")
    (source / "usr/local/bin/ciproxy-guest").write_bytes(b"fixture guest\n")
    (source / "privileged-tool").write_bytes(b"fixture tool\n")
    (source / "privileged-tool").chmod(0o755)
    subprocess.run(["chown", "1234:2345", str(source / "privileged-tool")], check=True)
    subprocess.run(["setfacl", "-m", "u:34567:r-x", str(source / "privileged-tool")], check=True)
    subprocess.run(["setcap", "cap_net_raw+ep", str(source / "privileged-tool")], check=True)
    os.link(source / "privileged-tool", source / "tool-alias")
    with (source / "sparse-data").open("wb") as target:
        target.seek(8388607)
        target.write(b"!")
    pins = dict(line.split("=", 1) for line in (image / "pins.env").read_text().splitlines()
                if line and not line.startswith("#"))
    report = work / PINNED_RECIPE_DIRECTORY / "images/ubuntu"
    report.mkdir(parents=True)
    (report / "software-report.json").write_text('{"fixture_source":"pinned-recipe"}\n')
    stale = work / ("runner-images-" + pins["RUNNER_IMAGES_COMMIT"]) / "images/ubuntu"
    stale.mkdir(parents=True)
    (stale / "software-report.json").write_text('{"fixture_source":"stale-unpinned-recipe"}\n')
    (work / "qemu-output").mkdir()
    (work / "qemu-output/ciproxy.qcow2").write_bytes(b"fixture source disk\n")
    (work / "upstream-software.json").write_text("{}\n")
    (work / "vmlinux").write_bytes(b"fixture kernel\n")
    tools = root / "tools"
    tools.mkdir()
    for name in ["virt-customize", "guestfish", "sudo"]:
        (tools / name).symlink_to(image / "_checks_/export_tools.py")
    environment = {**os.environ, "LC_ALL": "C",
                   "PATH": str(tools) + os.pathsep + os.environ["PATH"],
                   "CIPROXY_TEST_SOURCE": str(source),
                   "CIPROXY_TEST_EVENTS": str(root / "events.jsonl"),
                   "CIPROXY_TEST_FSTAB": SOURCE_FSTAB}
    environment.pop("ROOTFS_BYTES", None)
    return root, source, work, environment


def run(command, root, name, environment=None):
    result = subprocess.run(command, env=environment, capture_output=True, text=True)
    (root / (name + ".log")).write_text(result.stdout + result.stderr)
    if result.returncode:
        raise AssertionError(f"{name} exited {result.returncode}: {result.stdout}{result.stderr}")
    return result.stdout


def disk_file(image, path, root, name):
    output = root / (name + ".bin")
    run(["debugfs", "-R", f"dump {path} {output}", str(image)], root, name)
    return output.read_bytes()


def disk_attribute(image, path, attribute, root, name):
    output = root / (name + ".bin")
    run(["debugfs", "-R", f"ea_get -f {output} {path} {attribute}", str(image)], root, name)
    return output.read_bytes().hex()
