#!/usr/bin/env python3
"""Operator proof that launch rejects fresh and already issued credentials."""

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import sys
from urllib.error import HTTPError
from urllib.parse import quote
from urllib.request import Request, urlopen


ACCOUNTS = {
    "shared": "ciproxy-shared-preview-apply@ciproxy.iam.gserviceaccount.com",
    "trust": "ciproxy-trust-preview-apply@ciproxy.iam.gserviceaccount.com",
}


class ProbeError(ValueError):
    """A safe operator message with no provider payload or bearer value."""


def utc_now():
    return datetime.now(timezone.utc)


def request(url, token, body=None):
    data = None if body is None else json.dumps(body).encode()
    call = Request(url, data=data, headers={"Authorization": f"Bearer {token}",
                                         "Content-Type": "application/json"})
    try:
        with urlopen(call, timeout=30) as response:
            return response.status, json.load(response)
    except HTTPError as error:
        # Do not print request headers, response bodies, or bearer credentials.
        return error.code, {}


def mint(account, operator):
    return request(
        f"https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/{account}:generateAccessToken",
        operator, {"scope": ["https://www.googleapis.com/auth/cloud-platform"], "lifetime": "3600s"},
    )


def state_url(kind):
    prefix = "shared/shared" if kind == "shared" else "trust/seed"
    bucket = f"ciproxy-{kind}-tfstate-ciproxy"
    return f"https://storage.googleapis.com/storage/v1/b/{bucket}/o/{quote(prefix + '/default.tfstate', safe='')}"


def unexpired(document):
    expiry = datetime.fromisoformat(document["expireTime"].replace("Z", "+00:00"))
    if expiry <= utc_now():
        raise ProbeError("Captured token expired. It cannot prove revocation.")


def probe(mode, directory):
    operator = subprocess.check_output(["gcloud", "auth", "print-access-token"], text=True).strip()
    directory.mkdir(parents=True, exist_ok=True, mode=0o700)
    os.chmod(directory, 0o700)
    for kind, account in ACCOUNTS.items():
        path = directory / f"{kind}.json"
        if mode == "capture":
            code, document = mint(account, operator)
            if code != 200:
                raise ProbeError(f"Cannot capture {account}: token mint returned HTTP {code}.")
            unexpired(document)
            code, _ = request(state_url(kind), document["accessToken"])
            if code != 200:
                raise ProbeError(f"Prelaunch baseline state read failed: HTTP {code}.")
            # Atomic creation refuses to overwrite an earlier capture unnoticed.
            descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            with os.fdopen(descriptor, "w") as output:
                json.dump({**document, "account": account}, output)
            print(f"Captured {account}. Expires {document['expireTime']}. Baseline read passed.")
            continue
        document = json.loads(path.read_text())
        if document["account"] != account:
            raise ProbeError("Captured account does not match the launch account.")
        unexpired(document)
        code, status = request(f"https://iam.googleapis.com/v1/projects/ciproxy/serviceAccounts/{account}", operator)
        if code != 200 or status.get("disabled") is not True:
            raise ProbeError(f"{account} must report disabled=true before the proof.")
        code, _ = mint(account, operator)
        if code not in (400, 401, 403, 404):
            raise ProbeError(f"Fresh impersonation was not refused: HTTP {code}.")
        code, _ = request(state_url(kind), document["accessToken"])
        if code not in (401, 403):
            raise ProbeError(f"Existing unexpired token was not refused: HTTP {code}.")
        unexpired(document)
        print(f"Verified {account}: disabled, fresh impersonation refused, unexpired token refused.")


def main(arguments=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["capture", "verify"])
    parser.add_argument("directory", type=Path)
    args = parser.parse_args(arguments)
    try:
        probe(args.mode, args.directory)
    except ProbeError as error:
        print(f"Launch credential proof failed: {error}", file=sys.stderr)
        return 1
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        # Never echo third-party exceptions that might contain credentials.
        print(f"Launch credential proof failed: {type(error).__name__}. Check status, expiry, and operator permissions.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
