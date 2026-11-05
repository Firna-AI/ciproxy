#!/usr/bin/env python3
"""Select central promotion from the cloud launch latch and trusted event."""

import argparse
import json
import os
import subprocess
import sys

from promotion_event import check_checkout, validate


PRELAUNCH_ACCOUNT = "ciproxy-shared-preview-apply@ciproxy.iam.gserviceaccount.com"
PRELAUNCH_TRUST = "ciproxy-trust-preview-apply@ciproxy.iam.gserviceaccount.com"


def launched():
    """Account disablement, not an editable repository variable, is the latch."""
    states = []
    for account in (PRELAUNCH_ACCOUNT, PRELAUNCH_TRUST):
        result = subprocess.check_output([
            "gcloud", "iam", "service-accounts", "describe", account,
            "--project=ciproxy", "--format=json",
        ], text=True)
        status = json.loads(result)
        if type(status.get("disabled", False)) is not bool:
            raise ValueError("The prelaunch account has an invalid disabled status.")
        states.append(status.get("disabled", False))
    if states[0] != states[1]:
        raise ValueError("The GCP launch handoff is incomplete. Complete production bootstrap.")
    return states[0]


def routing(environment, production_ready, context):
    """The environment roots publish flags. Only the central owner changes APIs."""
    if context.get("PRODUCTION_DEPLOY_ENABLED") == "true" and not production_ready:
        raise ValueError("Production switch is on before the GCP launch handoff. Complete bootstrap first.")
    if environment == "production" and not production_ready:
        raise ValueError("Complete production bootstrap and revoke prelaunch access first.")
    owner = "production" if production_ready else "main-preview"
    shared_write = environment == owner
    return {
        "production_ready": production_ready,
        "shared_write": shared_write,
        "trust_write": shared_write and context.get("GITHUB_REF") == "refs/heads/main",
    }


def require_writer(environment, root, context):
    if root == "foundation":
        return
    selection = routing(environment, launched(), context)
    permission = "trust_write" if root in ("seed", "bootstrap") else "shared_write"
    if not selection[permission]:
        raise ValueError(f"{environment} reads central {root} after the launch handoff.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("environment", choices=["main-preview", "production"])
    parser.add_argument("source")
    args = parser.parse_args()
    try:
        validate(args.environment, args.source, os.environ)
        check_checkout(args.source)
        result = routing(args.environment, launched(), os.environ)
        lines = [f"{name}={str(value).lower()}" for name, value in result.items()]
        print("\n".join(lines))
        if os.environ.get("GITHUB_OUTPUT"):
            with open(os.environ["GITHUB_OUTPUT"], "a") as output:
                output.write("\n".join(lines) + "\n")
    except (ValueError, OSError, KeyError, subprocess.CalledProcessError) as error:
        print(f"Shared promotion: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
