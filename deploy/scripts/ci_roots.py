#!/usr/bin/env python3
"""Executable CI root routing. Called by plan and reusable apply workflows."""

import argparse
import os

from promotion_event import check_checkout, validate
from shared_routing import launched, routing


def roots(scope, selection):
    if scope == "foundation":
        return ["foundation"]
    if scope == "trust":
        return ["seed", "bootstrap"] if selection["trust_write"] else []
    return [{"shared": "shared", "edge": "edge"}[scope]] if selection["shared_write"] else []


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("environment", choices=["main-preview", "production"])
    parser.add_argument("scope", choices=["trust", "shared", "foundation", "edge"])
    args = parser.parse_args()
    source = os.environ["CIPROXY_SOURCE_SHA"]
    validate(args.environment, source, os.environ)
    check_checkout(source)
    selection = routing(args.environment, launched(), os.environ)
    selected = " ".join(roots(args.scope, selection))
    print(selected)
    if os.environ.get("GITHUB_OUTPUT"):
        with open(os.environ["GITHUB_OUTPUT"], "a") as output:
            output.write(f"roots={selected}\n")


if __name__ == "__main__":
    main()
