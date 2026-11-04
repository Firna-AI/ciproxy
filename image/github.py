#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Use host-side GitHub App credentials for image registration and evidence."""

import argparse
import base64
import json
import os
import pathlib
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file, code, message, headers, url):
        return None


class GitHub:
    def __init__(self, args):
        now = int(time.time())
        header = self.encode({"typ": "JWT", "alg": "RS256"})
        claims = self.encode({"iat": now - 60, "exp": now + 540, "iss": args.app_id})
        signing_input = header + b"." + claims
        signature = subprocess.run(
            ["openssl", "dgst", "-sha256", "-sign", str(args.app_key)],
            input=signing_input, capture_output=True, check=True).stdout
        jwt = (signing_input + b"." + base64.urlsafe_b64encode(signature).rstrip(b"=")).decode()
        self.opener = urllib.request.build_opener(NoRedirect)
        self.token = jwt
        token = self.request("POST", f"/app/installations/{args.installation_id}/access_tokens", {})
        self.token = token["token"]

    @staticmethod
    def encode(value):
        return base64.urlsafe_b64encode(json.dumps(value, separators=(",", ":")).encode()).rstrip(b"=")

    def request(self, method, route, body=None):
        request = urllib.request.Request(
            "https://api.github.com" + route,
            data=None if body is None else json.dumps(body).encode(), method=method,
            headers={"Authorization": "Bearer " + self.token,
                     "Accept": "application/vnd.github+json", "Content-Type": "application/json",
                     "X-GitHub-Api-Version": "2026-03-10"})
        with self.opener.open(request, timeout=60) as response:
            return json.load(response)


def registration(client, args):
    org = urllib.parse.quote(args.organization, safe="")
    organization = client.request("GET", "/orgs/" + org)
    answer = client.request("POST", f"/orgs/{org}/actions/runners/generate-jitconfig", {
        "name": args.runner_id, "runner_group_id": args.runner_group_id,
        "labels": [args.label], "work_folder": "_work"})
    value = {
        "configure": {"type": "configure_runner", "protocol_version": 1,
                      "runner_id": args.runner_id, "organization_id": str(organization["id"]),
                      "registration": answer["encoded_jit_config"], "cache": None},
        "exit_report_url": args.callback_url_file.read_text().strip(),
        "exit_report_token": args.sandbox_token_file.read_text().strip(),
    }
    descriptor = os.open(args.output, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "w") as output:
        json.dump(value, output)
        output.write("\n")
    print(json.dumps({"runner_id": args.runner_id, "github_runner_id": answer["runner"]["id"],
                      "sandbox_id": args.sandbox_id, "registration_file": str(args.output)}))


def result(client, args):
    repository = "/".join(urllib.parse.quote(part, safe="") for part in args.repository.split("/"))
    answer = client.request("GET", f"/repos/{repository}/actions/jobs/{args.job_id}")
    print(json.dumps({key: answer[key] for key in [
        "id", "run_id", "status", "conclusion", "runner_id", "runner_name", "html_url", "steps"]},
        indent=2))


def runner_support(client, args):
    org = urllib.parse.quote(args.organization, safe="")
    version = urllib.parse.quote(args.version, safe="")
    print(json.dumps(client.request("GET", f"/orgs/{org}/actions/runners/deprecations/{version}"),
                     indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app-id", required=True)
    parser.add_argument("--app-key", type=pathlib.Path, required=True)
    parser.add_argument("--installation-id", type=int, required=True)
    commands = parser.add_subparsers(dest="action", required=True)
    command = commands.add_parser("registration")
    command.add_argument("--organization", required=True)
    command.add_argument("--runner-group-id", type=int, required=True)
    command.add_argument("--runner-id", required=True)
    command.add_argument("--sandbox-id", required=True)
    command.add_argument("--label", default="ciproxy-image-smoke")
    command.add_argument("--callback-url-file", type=pathlib.Path, required=True)
    command.add_argument("--sandbox-token-file", type=pathlib.Path, required=True)
    command.add_argument("--output", type=pathlib.Path, required=True)
    command = commands.add_parser("result")
    command.add_argument("--repository", required=True)
    command.add_argument("--job-id", type=int, required=True)
    command = commands.add_parser("runner-support")
    command.add_argument("--organization", required=True)
    command.add_argument("--version", required=True)
    args = parser.parse_args()
    try:
        {"registration": registration, "result": result,
         "runner-support": runner_support}[args.action](GitHub(args), args)
    except urllib.error.HTTPError as error:
        print(f"GitHub returned HTTP {error.code}. The request was not retried.", file=sys.stderr)
        return 1
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError):
        print("GitHub operation failed. Check the host-side inputs and permissions.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
