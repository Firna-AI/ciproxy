#!/usr/bin/env python3
"""Select phase-owned roots, targets, and immutable workload artifacts."""

import argparse
import os

from promotion_event import check_checkout, validate
from shared_routing import launched, routing
from workload_snapshot import evidence

SCOPES = ("trust", "shared", "data", "foundation", "edge", "preview-data", "preview-edge")


def roots(scope, selection):
    if scope == "foundation":
        return ["foundation"]
    if scope == "trust":
        return ["seed", "bootstrap"] if selection["trust_write"] else []
    return [scope.removeprefix("preview-")] if selection["shared_write"] else []


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("environment", choices=["main-preview", "production"])
    parser.add_argument("scope", choices=SCOPES)
    args = parser.parse_args()
    source = os.environ["CIPROXY_SOURCE_SHA"]
    validate(args.environment, source, os.environ)
    check_checkout(source)
    if args.scope.startswith("preview-") and args.environment != "production":
        raise ValueError("Only production promotes the separate preview roots after launch.")
    selection = routing(args.environment, launched(), os.environ)
    target = "main-preview" if args.scope.startswith("preview-") else args.environment
    selected = roots(args.scope, selection)
    outputs = {"roots": " ".join(selected), "target": target}
    if selected and selected[0] in ("foundation", "edge"):
        outputs.update(evidence(target, args.environment))
    print(outputs["roots"])
    if os.environ.get("GITHUB_OUTPUT"):
        with open(os.environ["GITHUB_OUTPUT"], "a") as output:
            output.write("".join(f"{key}={value}\n" for key, value in outputs.items()))


if __name__ == "__main__":
    main()
