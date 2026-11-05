#!/usr/bin/env python3
"""Check a Terraform plan or print its exact destructive approval document."""

import argparse
import os
from pathlib import Path
import sys

from terraform_approval import PolicyError, format_approval, load_json, reason, require_committed
from terraform_roots import approval_path, configuration_digest, directory, scope, trust_resource


KEYS = {"schemaVersion", "environment", "configurationDigest", "reason", "actions"}
KNOWN = {"no-op", "create", "read", "update", "delete", "forget"}


def destructive_actions(plan):
    """Keep replacement order and deposed object keys from Terraform JSON."""
    if (not isinstance(plan, dict) or not isinstance(plan.get("format_version"), str)
            or plan["format_version"].split(".", 1)[0] != "1"
            or not isinstance(plan.get("resource_changes"), list)):
        raise PolicyError("Terraform plan JSON must contain resource_changes in format 1.")
    result, identities = [], set()
    for resource in plan["resource_changes"]:
        if not isinstance(resource, dict) or not isinstance(resource.get("change"), dict):
            raise PolicyError("Every resource change needs change details.")
        address, deposed = resource.get("address"), resource.get("deposed")
        actions = resource["change"].get("actions")
        if not isinstance(address, str) or not address:
            raise PolicyError("Every resource change needs a nonempty address.")
        if deposed is not None and (not isinstance(deposed, str) or not deposed):
            raise PolicyError("A deposed object needs a nonempty key.")
        if (address, deposed) in identities:
            raise PolicyError(f"Terraform plan repeats resource object {address}.")
        identities.add((address, deposed))
        if (not isinstance(actions, list) or not actions
                or not all(isinstance(action, str) and action in KNOWN for action in actions)):
            raise PolicyError(f"Terraform plan has invalid actions for {address}.")
        if {"delete", "forget"}.intersection(actions):
            item = {"address": address, "actions": actions}
            if deposed is not None:
                item["deposed"] = deposed
            result.append(item)
    return sorted(result, key=lambda item: (item["address"], item.get("deposed", "")))


def trust_violations(root, plan):
    destructive_actions(plan)
    return [resource["address"] for resource in plan["resource_changes"]
            if {"delete", "forget"}.intersection(resource["change"]["actions"])
            and trust_resource(root, resource)]


def approval_document(repository, environment, root, plan, explanation):
    actions = destructive_actions(plan)
    if not actions:
        raise PolicyError("The plan has no destructive actions to approve.")
    return {
        "schemaVersion": 1,
        "environment": scope(environment, root),
        "configurationDigest": configuration_digest(repository),
        "reason": reason(explanation),
        "actions": actions,
    }


def check(repository, environment, root, plan, ci=False):
    directory(environment, root)
    actions = destructive_actions(plan)
    violations = trust_violations(root, plan) if ci else []
    if violations:
        raise PolicyError("CI never deletes, replaces or forgets trust resource "
                          + ", ".join(violations) + "; use the operator recovery path.")
    path = approval_path(repository, environment, root)
    if not actions:
        if path.exists():
            raise PolicyError(f"Destructive approval is stale and must be removed: {path}")
        return
    if not path.is_file():
        raise PolicyError(f"Destructive approval is required at {path}")
    if ci:
        require_committed(repository, path)
    document = load_json(path)
    if not isinstance(document, dict) or set(document) != KEYS:
        raise PolicyError("Destructive approval fields do not match schema version 1.")
    if type(document["schemaVersion"]) is not int or document["schemaVersion"] != 1:
        raise PolicyError("Destructive approval schema does not match.")
    if document["environment"] != scope(environment, root):
        raise PolicyError("Destructive approval scope, environment or root does not match.")
    reason(document["reason"])
    if document["configurationDigest"] != configuration_digest(repository):
        raise PolicyError("Destructive approval does not match the Terraform configuration.")
    approved = document["actions"]
    if not isinstance(approved, list):
        raise PolicyError("Destructive approval actions must be an array.")
    for item in approved:
        if not isinstance(item, dict) or set(item) not in (
                {"address", "actions"}, {"address", "actions", "deposed"}):
            raise PolicyError("Each approval action must name address and actions.")
    if approved != actions:
        raise PolicyError("Destructive approval actions do not exactly match the plan.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["check", "approval"])
    parser.add_argument("environment", choices=["main-preview", "production"])
    parser.add_argument("root", choices=["seed", "bootstrap", "foundation", "edge"])
    parser.add_argument("plan", type=Path)
    parser.add_argument("--reason")
    args = parser.parse_args()
    repository = Path.cwd()
    try:
        plan = load_json(args.plan)
        if args.command == "check":
            check(repository, args.environment, args.root, plan,
                  ci=os.environ.get("GITHUB_ACTIONS") == "true")
        else:
            print(format_approval(approval_document(
                repository, args.environment, args.root, plan, args.reason)), end="")
            if trust_violations(args.root, plan):
                print("CI refuses this trust removal. Use operator recovery.", file=sys.stderr)
    except (PolicyError, ValueError) as error:
        print(f"Terraform plan policy: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
