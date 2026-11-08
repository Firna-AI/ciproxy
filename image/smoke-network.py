#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Prove public DNS and HTTPS on an image copy before any runner starts."""

import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import sys
import tempfile
import time
import uuid

sys.dont_write_bytecode = True

from _smoke_.network_host import run, running, stop


def probe(args):
    definition = Path(__file__).resolve().parent
    args.work_dir.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="network-", dir=args.work_dir)).resolve()
    vm = uuid.uuid4().hex
    unit = "ciproxy-netproof-" + vm + ".service"
    index = None
    root = work / "jailer/firecracker" / vm / "root"
    root.mkdir(parents=True)
    forward = run("sysctl", "-n", "net.ipv4.ip_forward").strip()
    print("Network evidence: " + str(work), flush=True)
    result = {"status": "FAIL", "runner_registration_supplied": False}
    try:
        prepared = json.loads(run(str(args.network_driver), "create"))
        index = prepared["index"]
        namespace = prepared["namespace"]
        uid = prepared["uid"]
        config = prepared["configuration"]
        run("cp", "--reflink=auto", "--sparse=auto", str(args.artifacts / "rootfs.ext4"), str(root / "rootfs.ext4"))
        if args.disk_bytes:
            if args.disk_bytes < (root / "rootfs.ext4").stat().st_size:
                raise RuntimeError("The probe disk cannot shrink the release")
            os.truncate(root / "rootfs.ext4", args.disk_bytes)
            run("resize2fs", str(root / "rootfs.ext4"))
        shutil.copyfile(args.artifacts / "vmlinux", root / "vmlinux")
        (root / "docker-slot").write_bytes(b"\0" * 4096)
        service = work / "network-proof.service"
        service.write_text("[Unit]\nAfter=network-online.target docker.service systemd-networkd.service systemd-resolved.service\n"
                           "Before=ciproxy-guest.service\nWants=network-online.target docker.service\n"
                           "[Service]\nType=oneshot\nExecStart=/usr/bin/python3 /usr/local/libexec/ciproxy-network-proof.py\n"
                           "[Install]\nWantedBy=multi-user.target\n")
        injection = work / "inject.debugfs"
        injection.write_text("write " + str(definition / "_smoke_/network_guest.py") + " /usr/local/libexec/ciproxy-network-proof.py\n"
                             "write " + str(service) + " /etc/systemd/system/ciproxy-network-proof.service\n"
                             "symlink /etc/systemd/system/multi-user.target.wants/ciproxy-network-proof.service "
                             "/etc/systemd/system/ciproxy-network-proof.service\n")
        if args.initialization_failure:
            invalid = work / "invalid-guest.env"
            invalid.write_text("IMAGE_ID=network-smoke\nVSOCK_PORT=invalid\n")
            with injection.open("a") as output:
                output.write("rm /etc/ciproxy/guest.env\nwrite " + str(invalid) + " /etc/ciproxy/guest.env\n")
        run("debugfs", "-w", "-f", str(injection), str(root / "rootfs.ext4"))
        (root / "config.json").write_text(json.dumps(config))
        (work / "vm-config.json").write_text(json.dumps(config, indent=2) + "\n")
        for path in root.iterdir():
            os.chown(path, uid, uid)
            path.chmod(0o600)
        run("systemd-run", "--quiet", "--collect", "--service-type=exec", "--unit", unit,
            "--property=Delegate=yes", "--property=KillMode=control-group",
            "--property=StandardOutput=append:" + str(work / "console.log"),
            "--property=StandardError=append:" + str(work / "console.log"),
            str(args.tools_dir / "jailer"), "--id", vm, "--exec-file", str(args.tools_dir / "firecracker"),
            "--uid", str(uid), "--gid", str(uid), "--cgroup-version", "2",
            "--chroot-base-dir", str(work / "jailer"), "--netns", "/run/netns/" + namespace,
            "--", "--config-file", "/config.json", "--api-sock", "/api.sock")
        deadline = time.monotonic() + 360
        while time.monotonic() < deadline:
            console = work / "console.log"
            if console.exists() and "CIPROXY_NETWORK_PROOF_READY" in console.read_text(errors="replace"):
                break
            time.sleep(1)
        else:
            raise RuntimeError("The guest network collector did not finish")
        if args.initialization_failure:
            deadline = time.monotonic() + 180
            while time.monotonic() < deadline and running(unit):
                time.sleep(1)
            if running(unit):
                raise RuntimeError("Guest initialization failure did not end the VMM")
            console = (work / "console.log").read_text(errors="replace")
            if "reboot: Restarting system" not in console:
                raise RuntimeError("The VMM ended without an observed guest reset")
            result["initialization_reset_observed"] = True
        else:
            run("systemctl", "stop", unit)
        report = json.loads(run("debugfs", "-R", "cat /var/lib/ciproxy-network-proof.json", str(root / "rootfs.ext4")))
        (work / "guest-network.json").write_text(json.dumps(report, indent=2) + "\n")
        if report["public_dns"]["exit_code"] != 0:
            raise RuntimeError("The guest could not resolve the public name before runner startup")
        https = report["public_https"]
        if https["exit_code"] != 0 or not https["stdout"].strip().startswith("2"):
            raise RuntimeError("The guest could not open checked public HTTPS before runner startup")
        result["status"] = "PASS"
        print("PASS: guest public DNS and HTTPS before runner startup", flush=True)
    except (OSError, ValueError, RuntimeError) as error:
        result["reason"] = str(error)
        print("FAIL: " + str(error), file=sys.stderr, flush=True)
    finally:
        remaining = [] if stop(unit) else ["VM process"]
        if index is not None and not remaining:
            try:
                run(str(args.network_driver), "remove", str(index))
            except RuntimeError as error:
                remaining.append(str(error))
        if remaining:
            result.update(status="FAIL", reason="Probe cleanup remains: " + ", ".join(remaining))
            print("FAIL: " + result["reason"], file=sys.stderr)
        if forward != "1":
            run("sysctl", "-qw", "net.ipv4.ip_forward=" + forward)
        if not remaining:
            shutil.rmtree(work / "jailer")
        (work / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    return 0 if result["status"] == "PASS" else 1


def main():
    definition = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifacts", type=lambda value: Path(value).resolve())
    parser.add_argument("--tools-dir", type=lambda value: Path(value).resolve(), required=True)
    parser.add_argument("--network-driver", type=lambda value: Path(value).resolve(), required=True,
                        help="compiled opt-in agent network-smoke-driver target")
    parser.add_argument("--work-dir", type=Path, default=Path(".context/image-network-smoke"))
    parser.add_argument("--disk-bytes", type=int, help="optional grown copy size from the agent sizing helper")
    parser.add_argument("--initialization-failure", action="store_true",
                        help="prove the image hook ends the VMM after an early guest failure")
    args = parser.parse_args()
    if os.geteuid() != 0 or not os.access("/dev/kvm", os.R_OK | os.W_OK):
        print("SKIP: root and usable KVM are required", file=sys.stderr)
        return 77
    return probe(args)


def interrupted(signum, frame):
    raise RuntimeError("The operator interrupted the network probe")


if __name__ == "__main__":
    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGINT, interrupted)
    sys.exit(main())
