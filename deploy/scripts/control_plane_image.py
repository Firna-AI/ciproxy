#!/usr/bin/env python3
"""Build without cloud credentials or publish an archive bound to its build."""

import argparse
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

from image_context import DOCKERFILE, PLATFORM, materialize, source_context
from image_outputs import outputs
from image_registry import REPOSITORY, archive_digest, immutable, lookup, publish


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
    outputs({"digest": archive_digest(args.archive)})


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
            command.add_argument("--expected-digest", default="")
    args = parser.parse_args()
    try:
        if hasattr(args, "source_hash") and not re.fullmatch("[0-9a-f]{64}", args.source_hash):
            raise ValueError("A SHA-256 source hash is required.")
        if args.command == "inputs":
            source_hash, _ = source_context(args.source, args.revision)
            outputs({"source_hash": source_hash})
        elif args.command == "build":
            build(args)
        elif args.command == "lookup":
            outputs({"image": lookup(args.source_hash, args.rollback)})
        elif args.command == "publish":
            if not args.existing and (args.archive is None or not args.expected_digest):
                raise ValueError("A built archive and the build job's image digest are required.")
            outputs({"image": immutable(args.existing) if args.existing else publish(
                args.source_hash, args.archive, args.expected_digest)})
    except ValueError as error:
        print(f"Control-plane image: {error}", file=sys.stderr)
        return 1
    except (KeyError, OSError, subprocess.CalledProcessError):
        print("Control-plane image operation failed. Check the source, bound preview evidence, or repository access.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
