#!/usr/bin/env python3
"""Check the trusted deployment event before using cloud credentials."""

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys

from terraform_roots import ENVIRONMENTS, PRIMARY, REPOSITORY


def validate(environment, source, context):
    """Use the preview event commit, never the workflow_run default-branch head."""
    if environment not in ENVIRONMENTS or not re.fullmatch("[0-9a-f]{40}", source):
        raise ValueError("A supported environment and full source commit are required.")
    if context.get("GITHUB_REPOSITORY") != REPOSITORY:
        raise ValueError("Deployment requires the Firna-AI/ciproxy repository.")
    event = json.loads(Path(context["GITHUB_EVENT_PATH"]).read_text())
    repository = event.get("repository", {})
    if repository.get("full_name") != REPOSITORY or repository.get("id") != 1442045837:
        raise ValueError("Deployment event repository does not match.")
    expected = f"{REPOSITORY}/.github/workflows/deploy-{environment}.yml@"
    ref = context.get("GITHUB_REF", "")
    if context.get("GITHUB_WORKFLOW_REF") != expected + ref:
        raise ValueError("Deployment requires its trusted environment workflow.")
    if environment == "main-preview":
        if context.get("GITHUB_EVENT_NAME") != "push" or ref not in (PRIMARY, "refs/heads/main"):
            raise ValueError("Main preview requires a push to the primary branch or main.")
        if event.get("after") != source or context.get("GITHUB_SHA") != source:
            raise ValueError("Main preview must deploy the push commit.")
        return
    if context.get("PRODUCTION_DEPLOY_ENABLED") != "true":
        raise ValueError("Production is off. Set PRODUCTION_DEPLOY_ENABLED=true at launch.")
    if context.get("GITHUB_EVENT_NAME") != "workflow_run" or ref != "refs/heads/main":
        raise ValueError("Production requires workflow_run on main.")
    run = event.get("workflow_run", {})
    if (run.get("name") != "Deploy main preview" or run.get("status") != "completed"
            or run.get("conclusion") != "success" or run.get("event") != "push"
            or run.get("head_branch") != "main"
            or run.get("path") != ".github/workflows/deploy-main-preview.yml"):
        raise ValueError("Production requires a successful main preview push run on main.")
    for key in ("repository", "head_repository"):
        upstream = run.get(key, {})
        if upstream.get("full_name") != REPOSITORY or upstream.get("id") != 1442045837:
            raise ValueError("Main preview run repository does not match.")
    if run.get("head_sha") != source:
        raise ValueError("Production must deploy the exact successful main preview commit.")


def check_checkout(source):
    actual = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    if actual != source:
        raise ValueError("Checkout does not match the approved deployment commit.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("environment", choices=ENVIRONMENTS)
    parser.add_argument("source")
    args = parser.parse_args()
    try:
        validate(args.environment, args.source, os.environ)
        check_checkout(args.source)
    except (ValueError, KeyError, OSError) as error:
        print(f"Promotion event: {error}", file=sys.stderr)
        return 1
    print(f"Accepted {args.environment} commit {args.source}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
