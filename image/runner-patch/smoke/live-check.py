#!/usr/bin/env python3
"""Run the official cache client through local executables on a GitHub runner."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True

from runtime import ROOT, Processes, start_cache


def original(name):
    value = os.environ.get("Z" + name[1:]) or os.environ.get(name)
    if not value:
        raise RuntimeError(f"missing original runner value for {name}")
    return value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registration", required=True, type=Path,
                        help="trusted Registration JSON for this runner assignment")
    parser.add_argument("--cache-module", required=True, type=Path,
                        help="installed official @actions/cache module directory")
    args = parser.parse_args()
    registration = json.loads(args.registration.read_text())
    if not all(registration.get("permissions", {}).get(mode) for mode in ("read", "write")):
        raise RuntimeError("this smoke needs a trusted job with cache read and write access")
    if registration["facts"]["event"].startswith("pull_request"):
        raise RuntimeError("run the live write smoke on a trusted default-branch job")
    if registration["facts"]["git_ref"] != registration["facts"]["default_ref"]:
        raise RuntimeError("the trusted registration must identify a default-branch job")
    registration["github_token"] = os.environ["ACTIONS_RUNTIME_TOKEN"]
    registration["cache_url"] = original("ACTIONS_CACHE_URL")
    registration["results_url"] = original("ACTIONS_RESULTS_URL")
    context = ROOT / ".context/ciproxy-mvp-m11"
    context.mkdir(parents=True, exist_ok=True)
    directory = Path(tempfile.mkdtemp(prefix="live-smoke-", dir=context))
    processes = Processes(directory)
    try:
        environment = start_cache(processes, registration)
        variables = dict(os.environ, CIPROXY_SMOKE_PROXY=environment["proxy"], CIPROXY_SMOKE_DIR=str(directory), CIPROXY_SMOKE_BYTES="16384")
        subprocess.run(["node", str(Path(__file__).with_name("sdk-check.mjs")), str(args.cache_module.resolve())], env=variables, check=True, cwd=ROOT)
        print("PASS: authenticated GitHub v2 round trip through machine service and VM proxy")
    finally:
        processes.close()
        (directory / "proxy.json").unlink(missing_ok=True)


if __name__ == "__main__":
    main()
