#!/usr/bin/env python3
"""Prove Docker disk recovery and fresh disks on a disposable KVM host."""

import argparse
import json
import os
from pathlib import Path
import sys

sys.dont_write_bytecode = True

from own_docker import lease, prepared
from own_smoke import Failure, Smoke
from docker_recovery_injection import mount_failure


def drain(smoke):
    smoke.action("drain")
    smoke.wait(lambda: not smoke.state()["runners"], "all prior VM registrations retire")


def offline_failure(smoke, base, original):
    area = base.parent.parent.parent
    smoke.command("systemctl", "stop", "ciproxy-agent.service")
    try:
        active = [json.loads(path.read_text()) for path in (area / "docker-leases").iterdir()
                  if path.suffix != ".pending"]
        if any(item["repository"] == original["repository"] for item in active):
            raise Failure("The source still has a running job lease")
        size = base.stat().st_size
        with base.open("r+b") as output:
            output.truncate(0)
            output.truncate(size)
            output.flush()
            os.fsync(output.fileno())
    finally:
        smoke.command("systemctl", "start", "ciproxy-agent.service")
    smoke.action("resume")
    return smoke.dispatch("docker")


def guest_failure(smoke, base):
    with mount_failure(smoke, base) as receipt:
        smoke.action("resume")
        run = smoke.dispatch("host")
        vm = smoke.local_job(run)
        prepared(smoke, vm)
        if not receipt.is_file() or json.loads(receipt.read_text())["vm"] != vm["local_id"]:
            raise Failure("The mount fault did not reach this VM's admitted writer disk")
        path = base.parent.parent.parent / "docker-leases" / vm["local_id"]
        held = json.loads(path.read_text())
        if held["phase"] != "Detached" or held["writer"] or int(held["bytes"]) <= 0:
            raise Failure("The failed mount did not retain its detached byte reservation")
        pid = smoke.pid(vm)
        smoke.command("systemctl", "restart", "ciproxy-agent.service")
        if json.loads(path.read_text()) != held or pid == "0" or smoke.pid(vm) != pid:
            raise Failure("Agent restart changed the reservation or stopped the running VM")
        smoke.record("failed_mount_reservation", {"vm": vm["local_id"], "bytes": held["bytes"],
                                                  "restart_preserved_vm": True})
    return run


def recover(smoke, fault):
    if not smoke.config.get("sticky_tier_ready"):
        raise Failure("Prepare Team or Business with enough spare Docker disk")
    marker = smoke.tag + "-recovery"
    first = smoke.dispatch("sticky-write", cache_key=marker)
    original = lease(smoke, smoke.local_job(first), True)
    smoke.finish(first)
    drain(smoke)
    area = (smoke.state_file.parent / "area").resolve()
    base = area / "docker" / original["repository"] / "base.ext4"
    failed = guest_failure(smoke, base) if fault == "guest" else offline_failure(smoke, base, original)
    failed_vm = smoke.local_job(failed)
    prepared(smoke, failed_vm)
    current = next(vm for vm in smoke.state()["runners"] if vm["local_id"] == failed_vm["local_id"])
    if current["preparation"]["Completed"]["docker_attached"]:
        raise Failure("The corrupt source did not select root Docker")
    if base.exists():
        raise Failure("The agent retained the corrupt repository disk")
    smoke.finish(failed)
    drain(smoke)
    smoke.action("resume")

    rebuilt = smoke.dispatch("sticky-write", cache_key=marker)
    replacement = lease(smoke, smoke.local_job(rebuilt), True)
    if replacement["repository"] != original["repository"] or not base.is_file():
        raise Failure("The next trusted job did not create a fresh repository disk")
    smoke.finish(rebuilt)
    reader = smoke.dispatch("sticky-read", cache_key=marker)
    lease(smoke, smoke.local_job(reader), True)
    smoke.finish(reader)
    smoke.record("docker_recovery", {"fault": fault, "corrupt_run": failed["id"], "rebuilt_run": rebuilt["id"],
                                     "reader_run": reader["id"], "root_fallback": True})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fault", choices=["guest", "offline"], default="guest")
    fault = parser.parse_args().fault
    if os.geteuid() != 0 or not os.access("/dev/kvm", os.R_OK | os.W_OK):
        print("SKIP: root and a usable KVM device are required", file=sys.stderr)
        return 77
    for name in ["CIPROXY_OWN_SMOKE_INPUT", "CIPROXY_SMOKE_CREDENTIALS"]:
        if not os.environ.get(name) or not Path(os.environ[name]).is_file():
            print("SKIP: set " + name + " to its private input file", file=sys.stderr)
            return 77
    smoke = None
    try:
        smoke = Smoke()
        smoke.evidence.rmdir()
        smoke.evidence = Path(".context/ciproxy-mvp/m25") / ("docker-recovery-" + smoke.tag)
        smoke.evidence.mkdir(parents=True)
        recover(smoke, fault)
        print("PASS: " + fault + " failure, root fallback, fresh disk, and subsequent persistence")
        print("Evidence: " + str(smoke.evidence))
        return 0
    except Failure as error:
        print("FAIL: " + str(error), file=sys.stderr)
        return 1
    except (OSError, ValueError, KeyError, TypeError, StopIteration):
        print("FAIL: operator input or local state could not be read", file=sys.stderr)
        return 1
    finally:
        if smoke:
            smoke.cleanup()
            if smoke.state()["mode"] == "Draining":
                smoke.action("resume")


if __name__ == "__main__":
    sys.exit(main())
