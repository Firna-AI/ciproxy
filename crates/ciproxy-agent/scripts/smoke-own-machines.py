#!/usr/bin/env python3
"""Run the M13 live checks on a disposable registered KVM machine."""

import argparse
import os
from pathlib import Path
import sys

sys.dont_write_bytecode = True

from own_smoke import Failure, Smoke
import own_scenarios
import own_cache
import own_docker


def main():
    scenarios = ["warm", "slots", "drain", "offline", "cache", "cache_isolation", "cache_fallback", "sticky_docker", "docker_no_fit", "restart", "disallowed", "remove"]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scenario", choices=["all", *scenarios], default="all")
    selected = parser.parse_args().scenario
    if os.geteuid() != 0 or not os.access("/dev/kvm", os.R_OK | os.W_OK):
        print("SKIP: root and a usable KVM device are required", file=sys.stderr)
        return 77
    for name in ["CIPROXY_OWN_SMOKE_INPUT", "CIPROXY_SMOKE_CREDENTIALS"]:
        if not os.environ.get(name) or not Path(os.environ[name]).is_file():
            print("SKIP: set " + name + " to its private input file", file=sys.stderr)
            return 77
    smoke = None
    skipped = []
    try:
        smoke = Smoke()
        for scenario in scenarios if selected == "all" else [selected]:
            print("RUN: " + scenario, flush=True)
            try:
                owner = own_cache if scenario.startswith("cache") else own_docker if scenario in ["sticky_docker","docker_no_fit"] else own_scenarios
                getattr(owner, scenario)(smoke)
                smoke.record("passed", {"scenario": scenario})
                print("PASS: " + scenario, flush=True)
            except own_scenarios.Skip as error:
                skipped.append(scenario)
                print("SKIP: " + str(error), flush=True)
        print("Evidence: " + str(smoke.evidence))
        return 77 if skipped else 0
    except Failure as error:
        print("FAIL: " + str(error), file=sys.stderr)
        return 1
    except (OSError, ValueError, KeyError, TypeError):
        print("FAIL: operator input or local state could not be read", file=sys.stderr)
        return 1
    finally:
        if smoke:
            smoke.cleanup()


if __name__ == "__main__":
    sys.exit(main())
