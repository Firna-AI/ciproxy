# SPDX-License-Identifier: Apache-2.0
"""Collect public network facts before the guest can configure a runner."""

import json
import os
from pathlib import Path
import subprocess


commands = {
    "addresses": ["ip", "-j", "address"],
    "routes": ["ip", "-j", "-4", "route", "show", "table", "all"],
    "networkctl": ["networkctl", "--no-pager", "status", "eth0"],
    "resolvectl": ["resolvectl", "--no-pager", "status"],
    "networkd_journal": ["journalctl", "-b", "-u", "systemd-networkd", "--no-pager", "-o", "short-monotonic"],
    "public_dns": ["timeout", "20", "getent", "ahostsv4", "github.com"],
    "public_https": ["curl", "--silent", "--show-error", "--connect-timeout", "5", "--max-time", "15",
                     "--output", "/dev/null", "--write-out", "%{http_code}", "https://github.com/"],
}
report = {"kernel_command_line": Path("/proc/cmdline").read_text().strip(),
          "runner_registration_supplied": False}
for name, command in commands.items():
    result = subprocess.run(command, capture_output=True, text=True, timeout=30)
    report[name] = {"exit_code": result.returncode, "stdout": result.stdout, "stderr": result.stderr}
with Path("/var/lib/ciproxy-network-proof.json").open("w") as output:
    json.dump(report, output, indent=2)
    output.flush()
    os.fsync(output.fileno())
subprocess.run(["sync"], check=True)
with open("/dev/ttyS0", "w") as serial:
    serial.write("CIPROXY_NETWORK_PROOF_READY\n")
    serial.flush()
