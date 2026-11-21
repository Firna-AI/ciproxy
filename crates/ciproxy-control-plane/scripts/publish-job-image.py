#!/usr/bin/env python3
"""Publish checked image artifacts under the deployment's /images path."""

import argparse
import os
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifacts", type=Path)
    parser.add_argument("bucket", help="Google Cloud Storage bucket name")
    parser.add_argument("release", help="New immutable release directory")
    args = parser.parse_args()
    credential_file = os.environ.get("CIPROXY_GCLOUD_CREDENTIALS_FILE")
    if not credential_file or not Path(credential_file).is_file():
        parser.error("Set CIPROXY_GCLOUD_CREDENTIALS_FILE to a private credential file.")
    if "/" in args.bucket or not args.bucket or args.bucket.startswith("-"):
        parser.error("Use a bucket name without gs:// or a path.")
    if not args.release or any(
        part in ("", ".", "..") for part in args.release.split("/")
    ):
        parser.error("Use a relative release directory without empty or dot parts.")
    repository = Path(__file__).resolve().parents[3]
    environment = os.environ.copy()
    environment["CLOUDSDK_AUTH_CREDENTIAL_FILE_OVERRIDE"] = str(
        Path(credential_file).resolve()
    )
    destination = f"gs://{args.bucket}/images/{args.release}"
    result = subprocess.run(
        ["bash", str(repository / "image/publish.sh"),
         str(args.artifacts.resolve()), destination, "--new-release"],
        env=environment,
        check=False,
    )
    if result.returncode:
        raise SystemExit(result.returncode)
    print(f"JOB_IMAGE_MANIFEST_PATH={args.release}/manifest.json")


if __name__ == "__main__":
    main()
