#!/usr/bin/env python3
"""Build without cloud credentials, publish, or select a bound workload image."""

import argparse
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

from image_context import DOCKERFILE, PLATFORM, materialize, source_context
from image_registry import REPOSITORY, immutable, lookup, publish
from promotion_event import check_checkout, validate
from workload_snapshot import evidence, load


def outputs(values):
    for key, value in values.items():
        print(f"{key}={value}")
    if os.environ.get("GITHUB_OUTPUT"):
        with open(os.environ["GITHUB_OUTPUT"], "a") as output:
            for key, value in values.items():
                output.write(f"{key}={value}\n")


def build(args):
    source_hash, files = source_context(args.source, args.revision)
    if source_hash != args.source_hash:
        raise ValueError("Build inputs differ from the registry lookup. Start a new run.")
    args.archive.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="ciproxy-image-") as temporary:
        context = Path(temporary) / "source"
        materialize(files, context)
        private_config = Path(temporary) / "docker"
        private_config.mkdir(mode=0o700)
        environment = dict(os.environ, DOCKER_CONFIG=str(private_config))
        subprocess.run([
            "buildctl", "build", "--frontend", "dockerfile.v0",
            "--local", f"context={context}",
            "--local", f"dockerfile={context / Path(DOCKERFILE).parent}",
            "--opt", "filename=Dockerfile", "--opt", f"platform={PLATFORM}",
            "--opt", f"build-arg:BUILD_REVISION={args.revision}",
            "--output", f"type=docker,name={REPOSITORY}:source-{source_hash},dest={args.archive.resolve()}",
        ], check=True, env=environment)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("inputs", "build"):
        command = commands.add_parser(name)
        command.add_argument("--source", type=Path, required=True)
        command.add_argument("--revision", required=True)
        if name == "build":
            command.add_argument("--source-hash", required=True)
            command.add_argument("--archive", type=Path, required=True)
    for name in ("lookup", "publish"):
        command = commands.add_parser(name)
        command.add_argument("--source-hash", required=True)
        if name == "lookup":
            command.add_argument("--rollback", default="")
        else:
            command.add_argument("--archive", type=Path)
            command.add_argument("--existing", default="")
    commands.add_parser("preview-evidence")
    commands.add_parser("promote")
    args = parser.parse_args()
    try:
        if hasattr(args, "source_hash") and not re.fullmatch("[0-9a-f]{64}", args.source_hash):
            raise ValueError("A SHA-256 source hash is required.")
        if args.command == "inputs":
            source_hash, _ = source_context(args.source, args.revision)
            trusted = Path(__file__).resolve().parents[2]
            tools = subprocess.check_output(["git", "-C", str(trusted), "rev-parse", "HEAD"], text=True).strip()
            outputs({"source_hash": source_hash, "tools_sha": tools})
        elif args.command == "build":
            build(args)
        elif args.command == "lookup":
            outputs({"image": lookup(args.source_hash, args.rollback)})
        elif args.command == "publish":
            if not args.existing and args.archive is None:
                raise ValueError("A built archive is required for publication.")
            outputs({"image": immutable(args.existing) if args.existing else publish(args.source_hash, args.archive)})
        else:
            source = os.environ.get("CIPROXY_SOURCE_SHA", "")
            validate("production", source, os.environ)
            check_checkout(source)
            if args.command == "preview-evidence":
                outputs(evidence("main-preview", "production"))
            else:
                inputs = load("main-preview", "production")
                outputs({"image": immutable(inputs.get("control_plane_image", ""))})
    except ValueError as error:
        print(f"Control-plane image: {error}", file=sys.stderr)
        return 1
    except (KeyError, OSError, subprocess.CalledProcessError):
        print("Control-plane image operation failed. Check the source, bound preview evidence, or repository access.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
