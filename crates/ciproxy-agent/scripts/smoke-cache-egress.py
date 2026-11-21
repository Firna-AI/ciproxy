#!/usr/bin/env python3
"""Prove the installed cache UID firewall on a disposable registered KVM host."""

import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys

sys.dont_write_bytecode = True

from cache_egress_host import Failure, InstalledCache, command, require
from cache_egress_lifecycle import firewall_fallback, lifecycle
from cache_egress_network import probes
import own_cache
from own_smoke import Failure as WorkflowFailure, Smoke


CHECKPOINT = Path(".context/agent-follow-ups/firewall/host-proof.json")


def prerequisites():
    if os.geteuid() != 0 or not os.access("/dev/kvm", os.R_OK | os.W_OK):
        return "root and a usable KVM device are required"
    for program in ["systemctl", "nft", "ip", "setpriv", "python3"]:
        if not shutil.which(program):
            return program + " is required"
    for variable in ["CIPROXY_OWN_SMOKE_INPUT", "CIPROXY_SMOKE_CREDENTIALS",
                     "CIPROXY_CACHE_ADMIN_TOKEN_FILE"]:
        if not os.environ.get(variable) or not Path(os.environ[variable]).is_file():
            return "set " + variable + " to its private input file"
    config = json.loads(Path(os.environ["CIPROXY_OWN_SMOKE_INPUT"]).read_text())
    if config.get("cache_firewall_disposable") is not True:
        return "confirm cache_firewall_disposable in the private input file"
    if not Path("/etc/ciproxy-cache-firewall").is_file():
        return "install the matching agent and cache service first"
    return None


def interrupted(signum, frame):
    raise Failure("The operator interrupted the firewall proof")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--after-boot", action="store_true",
                        help="finish the proof after rebooting this same disposable host")
    after_boot = parser.parse_args().after_boot
    smoke = None
    try:
        missing = prerequisites()
        if missing:
            print("SKIP: " + missing, file=sys.stderr)
            return 77
        boot = Path("/proc/sys/kernel/random/boot_id").read_text().strip()
        if after_boot:
            if not CHECKPOINT.is_file():
                print("SKIP: run the first phase before reboot", file=sys.stderr)
                return 77
            saved = json.loads(CHECKPOINT.read_text())
            require(saved["phase"] == "ready-for-reboot", "The first proof phase is incomplete")
            if saved["boot_id"] == boot:
                print("SKIP: reboot this host before --after-boot", file=sys.stderr)
                return 77
        smoke = Smoke()
        require(not any(vm["report"]["state"]["state"] == "running"
                        for vm in smoke.state()["runners"]),
                "Wait for all existing jobs before this disposable host proof")
        cache = InstalledCache()
        cache.inspect()
        command("systemctl", "is-enabled", "--quiet", "ciproxy-cache.service")
        print("RUN: installed cache UID and network policy", flush=True)
        observed = probes(cache)
        smoke.record("cache_firewall_network", {"phase": "after-boot" if after_boot else "initial",
                                                **observed})
        print("RUN: real VM cache save and restore", flush=True)
        own_cache.cache(smoke)
        if after_boot:
            saved.update({"phase": "passed", "reboot_observed": True,
                          "after_boot_evidence": str(smoke.evidence)})
            CHECKPOINT.write_text(json.dumps(saved) + "\n")
            print("PASS: installed cache firewall, real VM cache jobs, and reboot protection")
            return 0
        print("RUN: protected service starts and automatic restart", flush=True)
        lifecycle(cache)
        smoke.record("cache_firewall_starts", {"direct_start": True, "automatic_restart": True,
                                               "failed_policy_stays_closed": True})
        print("RUN: real GitHub cache restore during firewall failure", flush=True)
        firewall_fallback(smoke, cache)
        CHECKPOINT.parent.mkdir(parents=True, exist_ok=True)
        CHECKPOINT.write_text(json.dumps({"phase": "ready-for-reboot", "boot_id": boot,
                                         "initial_evidence": str(smoke.evidence)}) + "\n")
        print("SKIP: first phase passed; reboot this host, then run --after-boot")
        return 77
    except (Failure, WorkflowFailure) as error:
        print("FAIL: " + str(error), file=sys.stderr)
        return 1
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError):
        print("FAIL: local state, private input, or a network probe could not be read", file=sys.stderr)
        return 1
    finally:
        if smoke:
            smoke.cleanup()


if __name__ == "__main__":
    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGINT, interrupted)
    sys.exit(main())
