#!/usr/bin/env python3
"""Create project state and trust once. Production bootstrap retires preview authority."""

import argparse
import json
from pathlib import Path
import shutil
import subprocess

from terraform_roots import ENVIRONMENTS, ROOTS, backend


def run(*arguments):
    subprocess.run(arguments, check=True)


def bootstrap(environment, existing_state=False, runtime_uids=None):
    project = ENVIRONMENTS[environment]
    repository = Path.cwd()
    scratch = repository / ".context/deploy" / project
    scratch.mkdir(parents=True, exist_ok=True, mode=0o700)
    seed = backend(environment, "seed")
    run("gcloud", "services", "enable", "storage.googleapis.com", "serviceusage.googleapis.com",
        "cloudresourcemanager.googleapis.com", f"--project={project}")
    backup = scratch / "seed.tfstate"
    if existing_state:
        run("gcloud", "storage", "objects", "describe",
            f"gs://{seed.bucket}/{seed.prefix}/default.tfstate", "--format=value(generation)")
    else:
        local = scratch / "seed-local"
        local.mkdir(exist_ok=True)
        remote = repository / seed.directory
        source = (remote / "main.tf").read_text()
        (local / "main.tf").write_text(source.replace('  backend "gcs" {}\n', ""))
        shutil.copyfile(remote / ".terraform.lock.hcl", local / ".terraform.lock.hcl")
        backup = scratch / "seed.tfstate"
        local_backend = local / ".terraform/terraform.tfstate"
        migrated = local_backend.exists() and json.loads(local_backend.read_text()).get("backend", {}).get("type") == "gcs"
        if not migrated:
            run("terraform", f"-chdir={local}", "init", "-input=false", "-lockfile=readonly")
            run("terraform", f"-chdir={local}", "apply", f"-var=project_id={project}")
            shutil.copyfile(local / "terraform.tfstate", backup)
        (local / "main.tf").write_text(source)
        migration = ["-reconfigure", "-input=false"] if migrated else ["-migrate-state"]
        run("terraform", f"-chdir={local}", "init", *migration, "-lockfile=readonly", *seed.arguments())
    # Prepare central backends and both environment flag backends. Plan cannot
    # create state. Reconfiguration preserves an existing state on repeated calls.
    targets = {(backend(env, root).bucket, backend(env, root).prefix): backend(env, root)
               for env in ENVIRONMENTS for root in ROOTS}
    for target in targets.values():
        run("terraform", f"-chdir={target.directory}", "init", "-reconfigure",
            "-input=false", "-lockfile=readonly", *target.arguments())
    root = backend(environment, "bootstrap").directory
    outputs = json.loads(subprocess.check_output([
        "terraform", f"-chdir={root}", "output", "-json",
    ], text=True))
    previous = outputs.get("production_enabled", {}).get("value", False)
    uids = outputs.get("runtime_service_account_uids", {}).get("value", {})
    if runtime_uids:
        uids.update(json.loads(runtime_uids.read_text())["runtime_service_account_uids"])
    inputs = scratch / "runtime-uids.tfvars.json"
    inputs.write_text(json.dumps({"runtime_service_account_uids": uids}, indent=2) + "\n")
    # A preview setup after launch must NEVER enable retired accounts/providers.
    production_enabled = environment == "production" or previous
    run("terraform", f"-chdir={root}", "apply", f"-var=project_id={project}",
        f"-var=production_enabled={str(production_enabled).lower()}", f"-var-file={inputs}")
    if production_enabled:
        for account in ("ciproxy-shared-plan", "ciproxy-shared-preview-apply", "ciproxy-trust-preview-apply"):
            status = json.loads(subprocess.check_output([
                "gcloud", "iam", "service-accounts", "describe",
                f"{account}@ciproxy.iam.gserviceaccount.com", "--project=ciproxy", "--format=json",
            ], text=True))
            if status.get("disabled") is not True:
                raise ValueError(f"Launch handoff incomplete: {account} is not disabled.")
        print("Prelaunch shared plan/apply and trust accounts are disabled. Existing tokens are rejected.")
        print("Verify revocation and propagation before enabling PRODUCTION_DEPLOY_ENABLED.")
    run("terraform", f"-chdir={root}", "output", "-json", "github_environment_variables")
    if backup.exists():
        print(f"Back up {backup} outside Git.")
    print("Set the printed GitHub identity variables.")
    print("Create the first shared core as operator, then capture namespace ServiceAccount UIDs before CI.")
    print("Follow deploy/runtime-identity.md. Later protected-main trust and shared promotions stay automatic.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("environment", choices=ENVIRONMENTS)
    parser.add_argument("--existing-state", action="store_true",
                        help="Use existing project state without another local seed apply.")
    parser.add_argument("--runtime-uids", type=Path,
                        help="Operator-captured UID map. Merge into protected bootstrap inputs.")
    args = parser.parse_args()
    bootstrap(args.environment, args.existing_state, args.runtime_uids)
