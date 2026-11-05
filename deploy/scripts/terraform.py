#!/usr/bin/env python3
"""Save, check, or apply the exact Terraform plan for one environment root."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

from promotion_event import check_checkout, validate
from terraform_approval import load_json
from terraform_policy import check
from terraform_roots import ENVIRONMENTS, ROOTS, backend, configuration_digest
from shared_routing import launched, require_writer
from workload_snapshot import load as workload_inputs


def command(root, *arguments, capture=False):
    return subprocess.run(
        ["terraform", f"-chdir={root}", *arguments], check=True, text=True,
        stdout=subprocess.PIPE if capture else None,
    ).stdout


def variables(environment, root):
    if root == "seed":
        return {"project_id": ENVIRONMENTS[environment]}
    if root == "bootstrap":
        return {"project_id": ENVIRONMENTS[environment], "production_enabled": launched()}
    if root == "shared":
        return {"production_enabled": launched()}
    if root == "edge":
        return {}
    return workload_inputs(environment)


def metadata(repository, environment, root, binary):
    return {
        "environment": environment, "root": root,
        "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
        "run": os.environ.get("GITHUB_RUN_ID", ""),
        "attempt": os.environ.get("GITHUB_RUN_ATTEMPT", ""),
        "configurationDigest": configuration_digest(repository),
        "planDigest": "sha256:" + hashlib.sha256(binary.read_bytes()).hexdigest(),
    }


def promote(action, environment, root):
    repository = Path.cwd()
    target = backend(environment, root)
    location = target.directory
    scratch = repository / ".context/plans" / environment / root
    scratch.mkdir(parents=True, exist_ok=True, mode=0o700)
    binary = scratch / "release.tfplan"
    manifest = scratch / "metadata.json"
    ci = os.environ.get("GITHUB_ACTIONS") == "true"
    if ci:
        source = os.environ.get("CIPROXY_SOURCE_SHA", "")
        validate(environment, source, os.environ)
        check_checkout(source)
        if not os.environ.get("GITHUB_RUN_ID") or not os.environ.get("GITHUB_RUN_ATTEMPT"):
            raise ValueError("CI plans need the workflow run and attempt IDs.")
        if action == "apply" and os.environ.get("CIPROXY_REUSABLE_APPLY") != "true":
            raise ValueError("CI apply requires the reusable apply workflow.")
    if action != "plan":
        if not binary.is_file() or not manifest.is_file():
            raise ValueError("Run a new plan before checking or applying.")
        if load_json(manifest) != metadata(repository, environment, root, binary):
            raise ValueError("Saved plan is stale or changed. Start a new run with all jobs.")
    else:
        manifest.unlink(missing_ok=True)
    if ci:
        require_writer(environment, root, os.environ)
    command(location, "init", "-input=false", "-reconfigure", "-lockfile=readonly",
            *target.arguments())
    if action == "plan":
        inputs = scratch / "inputs.tfvars.json"
        inputs.write_text(json.dumps(variables(environment, root)))
        with (scratch / "plan.txt").open("w") as output:
            result = subprocess.run(
                ["terraform", f"-chdir={location}", "plan", "-input=false", "-no-color",
                 "-detailed-exitcode", "-lock-timeout=5m", f"-var-file={inputs}", f"-out={binary}"],
                stdout=output, stderr=subprocess.STDOUT, check=False,
            )
        print((scratch / "plan.txt").read_text(), end="")
        if result.returncode not in (0, 2):
            raise ValueError(f"Terraform plan failed with status {result.returncode}. Start a new plan.")
        manifest.write_text(json.dumps(metadata(repository, environment, root, binary), indent=2) + "\n")
    # JSON shipped with an artifact is evidence only. Always derive policy input
    # from the binary that Terraform will apply, including at the apply boundary.
    plan = command(location, "show", "-json", str(binary), capture=True)
    plan_path = scratch / "plan.json"
    plan_path.write_text(plan)
    check(repository, environment, root, load_json(plan_path), ci=ci)
    if action == "apply":
        command(location, "apply", "-input=false", "-lock-timeout=5m", str(binary))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["plan", "check", "apply"])
    parser.add_argument("environment", choices=ENVIRONMENTS)
    parser.add_argument("root", choices=ROOTS)
    args = parser.parse_args()
    try:
        promote(args.action, args.environment, args.root)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f"Terraform promotion: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
