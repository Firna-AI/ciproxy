# SPDX-License-Identifier: Apache-2.0
"""Adapt provider setup while retaining every shared software installer."""

import pathlib

from boat_build_cleanup import adapt_cleanup
from boat_build_owners import adapt_owners
from boat_build_setup import merge_directory
from boat_build_tmp import relocate_temp_paths


def adapt(upstream):
    scripts = upstream / "images/ubuntu/scripts/build"
    changes = []
    changes.append({"environment": {"TAR_OPTIONS": "--no-overwrite-dir"},
                    "reason": "GNU tar keeps the owner and mode of existing provider directories during native extraction, including CMake. New directories keep the archive metadata."})

    def edit(name, old, new, reason):
        path = scripts / name
        content = path.read_text()
        if old not in content:
            raise RuntimeError(f"Pinned native adaptation does not match {name}")
        path.write_text(content.replace(old, new))
        changes.append({"script": name, "reason": reason})

    environment = (scripts / "configure-environment.sh").read_text()
    start = environment.index("# Change waagent entries")
    end = environment.index("# Add localhost alias", start)
    edit("configure-environment.sh", environment[start:end], "",
         "Boat owns its resource disks, swap and cloud-init boot configuration; Azure resource-disk setup does not apply.")
    environment = (scripts / "configure-environment.sh").read_text()
    start = environment.index("# Relax root filesystem durability")
    end = environment.index("# Create symlink for tests", start)
    edit("configure-environment.sh", environment[start:end], "",
         "Boat owns the boot loader and kernel arguments; Firecracker/QEMU root mount setup does not apply.")
    edit("configure-environment.sh", "mkdir $AGENT_TOOLSDIRECTORY", "mkdir -p $AGENT_TOOLSDIRECTORY",
         "Boat can already have a toolcache directory; retain its contents.")
    edit("configure-environment.sh", "ln -s $HELPER_SCRIPTS/invoke-tests.sh /usr/local/bin/invoke_tests",
         "ln -sfn $HELPER_SCRIPTS/invoke-tests.sh /usr/local/bin/invoke_tests",
         "Refresh the owned test-helper link when changed source inputs require replay.")
    edit("configure-apt-sources.sh", "touch /etc/apt/apt-mirrors.txt", ": > /etc/apt/apt-mirrors.txt",
         "Recreate the owned shared mirror list before its fixed entries so replay keeps priorities unique.")
    edit("configure-system.sh", "mv -f /imagegeneration/post-generation /opt",
         merge_directory("/imagegeneration/post-generation", "/opt/post-generation"),
         "Merge refreshed post-generation files while retaining existing metadata on a replay.")
    changes.extend(adapt_cleanup(upstream))
    edit("install-pypy.sh", '    local package_url=$1\n',
         '    local package_url=$1\n'
         '    if [ -f "$AGENT_TOOLSDIRECTORY/PyPy/$toolset_version/x64.complete" ]; then\n'
         '        return 0\n'
         '    fi\n',
         "Reuse complete native PyPy versions without repeating their download or dependency installation.")
    edit("install-pypy.sh", '    mkdir $pypy_toolcache_version_path\n',
         '    rm -rf "$pypy_toolcache_version_path"\n    mkdir -p "$pypy_toolcache_version_path"\n',
         "Replace only the incomplete native PyPy version before retrying its installation.")
    for name in ("configure-system.sh", "install-nodejs.sh"):
        content = (scripts / name).read_text()
        for folder in ("/usr/share", "/opt", "/usr/local/bin", "/usr/local/lib/node_modules"):
            old = "chmod -R 777 " + folder
            if old in content:
                edit(name, old, "python3 /opt/ciproxy-build/definition/boat-build-permissions.py " + folder,
                     "Apply writable tool permissions to new paths and retain permissions of pre-existing provider paths.")
    edit("install-docker.sh", 'groupmod -g "$gid" docker',
         'groupmod -g "$(cat /etc/ciproxy/provider-docker-gid)" docker',
         "Preserve the provider Docker group number used by its already running controller and desktop processes.")
    edit("install-docker.sh", '# Docker daemon takes time',
         'systemctl daemon-reload\nsystemctl stop docker.socket docker.service\n'
         'systemctl restart containerd.service\nsystemctl start docker.socket docker.service\n\n'
         '# Docker daemon takes time',
         "Run installed Docker and containerd binaries before their upstream software tests.")
    tests = upstream / "images/ubuntu/scripts/tests/System.Tests.ps1"
    content = tests.read_text()
    for context, reason in [
        ("Azure resource disk mount timeout", "Boat owns its resource disk and cloud-init setup."),
        ("Root filesystem performance options", "Boat owns its boot loader, kernel arguments and initial root mount."),
    ]:
        old = f'Describe "{context}" {{'
        content = content.replace(old, f'Describe "{context}" -Skip:$true {{')
        changes.append({"test": "System.Tests.ps1: " + context, "reason": reason})
    old = 'It "All SCSI and NVMe devices have read_ahead_kb set to 128" {'
    content = content.replace(old, 'It "All SCSI and NVMe devices have read_ahead_kb set to 128" '
                              '-Skip:(-not (Get-ChildItem /sys/block/sd*, /sys/block/nvme*n* -ErrorAction SilentlyContinue)) {')
    changes.append({"test": "System.Tests.ps1: All SCSI and NVMe devices have read_ahead_kb set to 128",
                    "reason": "Boat uses a Virtio disk; this assertion applies only when SCSI or NVMe devices exist. The rule tests still run."})
    tests.write_text(content)
    changes.append(adapt_owners(upstream))
    for path in scripts.iterdir():
        if path.name.startswith(("install-", "Install-", "Configure-Toolset")):
            path.write_text(relocate_temp_paths(path.read_text()))
    helper = upstream / "images/ubuntu/scripts/helpers/install.sh"
    helper.write_text(relocate_temp_paths(helper.read_text()))
    changes.append({"scripts": "Software installers and download helper",
                    "reason": "Build downloads and extraction use the owned build temporary directory; finalization can remove them without deleting provider temporary files."})
    return changes
