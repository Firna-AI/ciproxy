#!/usr/bin/env python3
"""Create the state bucket locally, migrate it, and create CI trust once."""

import argparse
import json
from pathlib import Path
import shutil
import subprocess

from terraform_roots import ENVIRONMENTS


def run(*arguments):
    subprocess.run(arguments, check=True)


def bootstrap(environment):
    project = ENVIRONMENTS[environment]
    repository = Path.cwd()
    scratch = repository / ".context/deploy" / environment
    scratch.mkdir(parents=True, exist_ok=True, mode=0o700)
    bucket = f"ciproxy-{environment}-tfstate-{project}"
    run("gcloud", "services", "enable", "storage.googleapis.com", "serviceusage.googleapis.com",
        "cloudresourcemanager.googleapis.com", f"--project={project}")

    # Work in an isolated directory: the tracked seed root always has a GCS
    # backend, but the bucket cannot host its own state before it exists.
    local = scratch / "seed-local"
    local.mkdir(exist_ok=True)
    remote = repository / "deploy/terraform/seed"
    source = (remote / "main.tf").read_text()
    (local / "main.tf").write_text(source.replace('  backend "gcs" {}\n', ""))
    shutil.copyfile(remote / ".terraform.lock.hcl", local / ".terraform.lock.hcl")
    backup = scratch / "seed.tfstate"
    backend = local / ".terraform/terraform.tfstate"
    migrated = backend.exists() and json.loads(backend.read_text()).get("backend", {}).get("type") == "gcs"
    if not migrated:
        # An existing local state from the former bootstrap script is adopted
        # without trying to create the state bucket a second time.
        if backup.exists() and not (local / "terraform.tfstate").exists():
            shutil.copyfile(backup, local / "terraform.tfstate")
        run("terraform", f"-chdir={local}", "init", "-input=false", "-lockfile=readonly")
        run("terraform", f"-chdir={local}", "apply",
            f"-var=environment={environment}", f"-var=project_id={project}")
        shutil.copyfile(local / "terraform.tfstate", backup)
    (local / "main.tf").write_text(source)
    migration = ["-reconfigure", "-input=false"] if migrated else ["-migrate-state"]
    # Let Terraform show and confirm a migration. Never force-copy an old
    # local backup over a newer destination state on another machine.
    run("terraform", f"-chdir={local}", "init", *migration, "-lockfile=readonly",
        f"-backend-config=bucket={bucket}", f"-backend-config=prefix={environment}/seed")

    root = "deploy/terraform/bootstrap"
    run("terraform", f"-chdir={root}", "init", "-reconfigure", "-input=false", "-lockfile=readonly",
        f"-backend-config=bucket={bucket}", f"-backend-config=prefix={environment}/bootstrap")
    run("terraform", f"-chdir={root}", "apply",
        f"-var=environment={environment}", f"-var=project_id={project}")
    run("terraform", f"-chdir={root}", "output", "-json", "github_environment_variables")
    print(f"Back up {backup} outside Git. CI now manages seed, bootstrap, foundation and edge.")
    print("Set GitHub variables, then start CI to create foundation before preparing the release image.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("environment", choices=ENVIRONMENTS)
    bootstrap(parser.parse_args().environment)
