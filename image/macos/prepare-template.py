#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Prepare the pinned upstream macOS template for an SSH build of our own VM.

The upstream template builds with Anka or with Packer's null builder over SSH.
This copy keeps every upstream provisioner, removes the Anka plugin and its
source, selects the null builder, keeps the pinned Xcode set in a toolset copy,
and appends the ciproxy steps: the runner and the guest. The facts phase of
build.sh removes the build state afterwards, because the build still needs
SSH for the facts.
"""

import argparse
import json
import pathlib
import re

TEMPLATE_NAME = "ciproxy-macos-26-arm64.pkr.hcl"
TOOLSET_NAME = "toolset-26-ciproxy.json"


def replace_once(text, old, new, label):
    if text.count(old) != 1:
        raise SystemExit(f"the upstream template changed: {label}")
    return text.replace(old, new)


def remove_block(text, header, label):
    """Remove one top-level HCL block that starts with header."""
    start = text.find(header)
    if start < 0 or text.find(header, start + 1) >= 0:
        raise SystemExit(f"the upstream template changed: {label}")
    depth, index = 0, text.index("{", start)
    while True:
        if text[index] == "{":
            depth += 1
        elif text[index] == "}":
            depth -= 1
            if depth == 0:
                return text[:start] + text[index + 1:].lstrip("\n")
        index += 1


CIPROXY_STEPS = '''
  provisioner "shell" {
    inline = ["mkdir -p /tmp/ciproxy"]
  }

  provisioner "file" {
    destination = "/tmp/ciproxy/"
    sources     = ["${var.ciproxy_inputs}/runner.tar.gz", "${var.ciproxy_inputs}/guest",
                   "${var.ciproxy_definition}/install-runner.sh"]
  }

  provisioner "shell" {
    environment_vars = ["RUNNER_SHA256=${var.ciproxy_runner_sha256}"]
    execute_command  = "chmod +x {{ .Path }}; sudo {{ .Vars }} {{ .Path }}"
    inline           = ["bash /tmp/ciproxy/install-runner.sh /tmp/ciproxy/runner.tar.gz",
                        "cd /tmp/ciproxy/guest && ./install.sh ${var.ciproxy_image_id}",
                        "defaults write /Library/Preferences/com.apple.loginwindow autoLoginUser -string ${var.vm_username}"]
  }
}
'''

CIPROXY_VARIABLES = '''
variable "ciproxy_inputs" {
  type = string
}

variable "ciproxy_definition" {
  type = string
}

variable "ciproxy_image_id" {
  type = string
}

variable "ciproxy_ssh_key" {
  type = string
}

variable "ciproxy_runner_sha256" {
  type = string
}
'''


AUTOLOGIN_STEP = '''provisioner "shell" {
    environment_vars = ["USERNAME=${var.vm_username}"]
    execute_command  = "chmod +x {{ .Path }}; sudo {{ .Vars }} {{ .Path }}"
    inline           = ["defaults write /Library/Preferences/com.apple.loginwindow autoLoginUser -string \\"$USERNAME\\"",
                        "test -s /etc/kcpassword"]
  }

  '''


CLOSE_WINDOWS_STEP = '''

  provisioner "shell" {
    pause_before = "30s"
    inline = ["ok=0; for i in $(seq 36); do n=$(osascript -e 'tell application \\"Finder\\" to close every window' -e 'tell application \\"Finder\\" to count windows' 2>/dev/null || true); if [ \\"$n\\" = 0 ]; then ok=$((ok + 1)); [ $ok -ge 4 ] && exit 0; else ok=0; fi; sleep 5; done; exit 1"]
  }'''


def close_finder_windows(text):
    """Close Finder's windows after each upstream restart. The Setup
    Assistant phase opened Finder's Utilities window, Finder restores it at
    each login, and the upstream configure-windows.sh fails on any open
    window. Finder restores it some seconds after login, so the step waits
    until Finder has no window at four checks in a row. Packer runs inline
    steps with "sh -e", so a failed osascript call must not end the loop.
    The upstream TCC step has allowed osascript to reach Finder."""
    marker = 'provisioner "shell" {'
    parts = text.split(marker)
    for index in range(1, len(parts)):
        end = parts[index].find("\n  }")
        if "shutdown -r now" in parts[index][:end]:
            end += len("\n  }")
            parts[index] = parts[index][:end] + CLOSE_WINDOWS_STEP + parts[index][end:]
    return marker.join(parts)


def keep_autologin(text):
    """Write the auto-login user again before each upstream restart. The
    setting of configure-autologin.sh was lost once before the first
    restart, and the guest then had no desktop session for the UI steps."""
    marker = 'provisioner "shell" {'
    parts = text.split(marker)
    for index in range(1, len(parts)):
        if "shutdown -r now" in parts[index][:parts[index].find("\n  }")]:
            parts[index - 1] += AUTOLOGIN_STEP
    return marker.join(parts)


def add_retries(text):
    """Retry each upstream shell step that expects no restart, twice. Over
    SSH, a step can lose its session in rare cases while softwareupdate
    works. The upstream install steps can run again."""
    marker = 'provisioner "shell" {'
    parts = text.split(marker)
    for index in range(1, len(parts)):
        block = parts[index][:parts[index].find("\n  }")]
        if "expect_disconnect" not in block:
            parts[index] = "\n    max_retries = 2" + parts[index]
    return marker.join(parts)


def prepare_template(source):
    text = remove_block(source, "packer {", "the packer block")
    text = remove_block(text, 'source "veertu-anka-vm-clone" "template"', "the Anka source")
    text = replace_once(text, 'default = "veertu-anka-vm-clone"', 'default = "null"',
                        "the builder_type default")
    text = replace_once(text, "toolsets/toolset-26.json", f"toolsets/{TOOLSET_NAME}",
                        "the toolset path")
    # Packer logs in with the build's SSH key. The upstream scripts still get
    # the password through their environment variables.
    text = replace_once(text, 'ssh_password = "${var.vm_password}"',
                        'ssh_private_key_file = "${var.ciproxy_ssh_key}"', "the SSH password")
    text = close_finder_windows(keep_autologin(add_retries(text)))
    text = text.rstrip()
    if not text.endswith("}"):
        raise SystemExit("the upstream template changed: the end of the build block")
    return text[:-1].rstrip() + "\n" + CIPROXY_STEPS + CIPROXY_VARIABLES


def prepare_toolset(source, links):
    """Keep only the pinned Xcode versions. The default gets its own runtimes."""
    toolset = json.loads(source)
    xcode = toolset["xcode"]
    versions = [entry for entry in xcode["arm64"]["versions"] if entry["link"] in links]
    if sorted(entry["link"] for entry in versions) != sorted(links):
        raise SystemExit(f"the toolset has no Xcode version for each of {links}")
    if xcode["default"] not in links:
        raise SystemExit(f"the pinned Xcode set has no default Xcode {xcode['default']}")
    # Upstream's default Xcode can share the runtimes of another version. When
    # no kept version installs runtimes, the default installs its own.
    if all(entry["install_runtimes"] == "none" for entry in versions):
        default = next(entry for entry in versions if entry["link"] == xcode["default"])
        default["install_runtimes"] = "default"
    xcode["arm64"]["versions"] = versions
    return json.dumps(toolset, indent=4) + "\n"


def check_xcode_pin(toolset, xip_name, sha256):
    xcode = json.loads(toolset)["xcode"]
    default = next(entry for entry in xcode["arm64"]["versions"]
                   if entry["link"] == xcode["default"])
    if f"Xcode-{default['version']}.xip" != xip_name or default["sha256"] != sha256:
        raise SystemExit("pins.env and the upstream toolset name different Xcode archives")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("upstream", type=pathlib.Path, help="the extracted runner-images tree")
    parser.add_argument("--template", required=True, help="the template path in the tree")
    parser.add_argument("--xcode-links", required=True, help="comma-separated Xcode links")
    parser.add_argument("--xcode-xip", required=True)
    parser.add_argument("--xcode-sha256", required=True)
    args = parser.parse_args()
    macos = args.upstream / "images/macos"
    source = (args.upstream / args.template).read_text()
    links = [link for link in re.split(r"[,\s]+", args.xcode_links) if link]
    toolset = prepare_toolset((macos / "toolsets/toolset-26.json").read_text(), links)
    check_xcode_pin(toolset, args.xcode_xip, args.xcode_sha256)
    (macos / "toolsets" / TOOLSET_NAME).write_text(toolset)
    (macos / "templates" / TEMPLATE_NAME).write_text(prepare_template(source))
    print(macos / "templates" / TEMPLATE_NAME)


if __name__ == "__main__":
    main()
