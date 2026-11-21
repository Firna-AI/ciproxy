#!/usr/bin/python3
"""One-use test-host fault between Docker sizing and the guest mount."""

import json
import os
from pathlib import Path
import subprocess
import sys


def inject(config, output):
    target = Path(sys.argv[-1])
    base = Path(config["base"])
    if target.name != "docker-job.ext4" or not target.is_file() or not base.is_file():
        return
    if not os.path.samefile(target, base):
        return
    vm_id = target.parent.parent.name
    state = json.loads(Path(config["state"]).read_text())
    record = next(item for item in state["runners"] if item["local_id"] == vm_id)
    lease_path = Path(config["area"]) / "docker-leases" / vm_id
    lease = json.loads(lease_path.read_text())
    if not lease["writer"] or lease["repository"] != base.parent.name:
        return
    values = dict(line.split(":", 1) for line in output.decode().splitlines() if ":" in line)
    free = int(values["Free blocks"].strip()) * int(values["Block size"].strip())
    if free < int(record["report"]["requested_size"]["disk_bytes"]):
        return
    size = target.stat().st_size
    if size != int(lease["bytes"]):
        return
    try:
        receipt = os.open(config["receipt"], os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    except FileExistsError:
        return
    with os.fdopen(receipt, "w") as report:
        with target.open("r+b") as disk:
            disk.truncate(0)
            disk.truncate(size)
            disk.flush()
            os.fsync(disk.fileno())
        json.dump({"vm": vm_id, "bytes": size}, report)
        report.flush()
        os.fsync(report.fileno())


def main():
    config = json.loads(Path(__file__).with_name("fault.json").read_text())
    result = subprocess.run([config["real"], *sys.argv[1:]], capture_output=True, check=False)
    if result.returncode == 0:
        inject(config, result.stdout)
    sys.stdout.buffer.write(result.stdout)
    sys.stderr.buffer.write(result.stderr)
    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
