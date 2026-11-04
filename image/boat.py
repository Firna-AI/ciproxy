#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Prepare, share, and smoke-test the common full rootfs with Boat's API."""

import argparse
import base64
import hashlib
import json
import os
import pathlib
import shlex
import sys
import time
import urllib.error
import urllib.parse
import urllib.request


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file, code, message, headers, url):
        return None


class Api:
    def __init__(self):
        self.base = os.environ.get("BOAT_API_BASE", "https://boat.dev/api/v1")
        self.key = os.environ["BOAT_API_KEY"]
        self.opener = urllib.request.build_opener(NoRedirect)

    def request(self, method, route, body=None, headers=None):
        data = None if body is None else json.dumps(body).encode()
        request = urllib.request.Request(
            self.base + route, data=data, method=method,
            headers={"Authorization": "Bearer " + self.key,
                     "Content-Type": "application/json", **(headers or {})})
        with self.opener.open(request, timeout=660) as response:
            return json.load(response)

    def command(self, sandbox, command, detached=False):
        return self.request("POST", f"/sandboxes/{sandbox}/commands", {
            "command": command, "timeoutSeconds": 600, "detached": detached})

    def upload(self, sandbox, source, destination):
        answer = self.request("POST", f"/sandboxes/{sandbox}/files/upload-url", {
            "path": destination, "ttlSeconds": 3600})
        with source.open("rb") as contents:
            upload = urllib.request.Request(
                answer["url"], data=contents, method="PUT",
                headers={"Content-Length": str(source.stat().st_size),
                         "Content-Type": "application/octet-stream"})
            with urllib.request.urlopen(upload, timeout=3600) as response:
                response.read()


def create(api, args):
    body = {"type": args.size, "name": "ciproxy-image-" + args.size,
            "noEnv": True, "monitoring": False, "snapshots": args.template,
            "ttlSeconds": 7200 if args.trial else 21600}
    if args.share_file:
        body["from"] = json.loads(args.share_file.read_text())["shareCode"]
    answer = api.request("POST", "/sandboxes", body)
    sandbox = answer["sandbox"]["id"]
    print(json.dumps({"sandbox_id": sandbox, "size": args.size}))


def prepare(api, args):
    checksum = hashlib.sha256()
    with args.archive.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            checksum.update(block)
    digest = checksum.hexdigest()
    api.upload(args.sandbox, args.archive, "/tmp/ciproxy-rootfs.tar.zst")
    api.upload(args.sandbox, pathlib.Path(__file__).with_name("boat-prepare.sh"),
               "/tmp/ciproxy-prepare.sh")
    api.upload(args.sandbox, pathlib.Path(__file__).with_name("rootfs-archive.sh"),
               "/tmp/rootfs-archive.sh")
    answer = api.command(args.sandbox,
                         "sudo bash /tmp/ciproxy-prepare.sh "
                         "/tmp/ciproxy-rootfs.tar.zst " + digest, detached=True)
    print(json.dumps({"process_id": answer["processId"], "archive_sha256": digest}))


def snapshot(api, args):
    check = api.command(args.sandbox, "test ! -e /opt/ciproxy/rootfs/opt/actions-runner/.runner "
                        "&& test ! -e /opt/ciproxy/rootfs/run/ciproxy/registration.json")
    if check["exitCode"] != 0:
        raise RuntimeError("The template has runner data; use a fresh sandbox")
    name = urllib.parse.quote(args.name, safe="")
    api.request("POST", "/saved-snapshots", {"sandboxId": args.sandbox, "name": args.name})
    deadline = time.monotonic() + 600
    while time.monotonic() < deadline:
        saved = api.request("GET", "/saved-snapshots/" + name)["snapshot"]
        if saved["status"] == "ready":
            share = api.request("POST", "/saved-snapshots/" + name + "/shares", {})
            descriptor = os.open(args.share_file, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            with os.fdopen(descriptor, "w") as target:
                json.dump(share, target, indent=2)
                target.write("\n")
            print(json.dumps({"snapshot": args.name, "share_file": str(args.share_file)}))
            return
        if saved["status"] == "failed":
            raise RuntimeError("Boat could not save the full image")
        time.sleep(2)
    raise TimeoutError("The snapshot is still saving; inspect it before retrying")


def run(api, args):
    if args.registration.stat().st_mode & 0o077:
        raise RuntimeError("The local registration file must have mode 0600")
    directory = api.command(args.sandbox, "mktemp -d /tmp/ciproxy.XXXXXXXX")
    path = directory["stdout"].strip() + "/registration.json"
    api.request("PUT", f"/sandboxes/{args.sandbox}/files", {
        "path": path, "content": base64.b64encode(args.registration.read_bytes()).decode(),
        "encoding": "base64"})
    command = "sudo bash /opt/ciproxy/rootfs/opt/ciproxy-image/boat-run.sh " + shlex.quote(path)
    answer = api.command(args.sandbox, command, detached=True)
    print(json.dumps({"sandbox_id": args.sandbox, "process_id": answer["processId"]}))


def status(api, args):
    route = f"/sandboxes/{args.sandbox}"
    if args.process:
        route += "/commands/" + str(args.process)
    answer = api.request("GET", route)
    if args.process:
        fields = ["processId", "status", "running", "exitCode", "stdout", "stderr"]
        output = {key: answer.get(key) for key in fields}
    else:
        sandbox = answer.get("sandbox", answer)
        output = {key: sandbox.get(key) for key in ["id", "state", "type"]}
    print(json.dumps(output, indent=2))


def metadata(api, args):
    command = ("sudo python3 /opt/ciproxy/rootfs/opt/ciproxy-image/metadata-smoke.py "
               "/opt/ciproxy/rootfs " + shlex.quote(args.file))
    answer = api.command(args.sandbox, command)
    if answer["exitCode"] != 0:
        raise RuntimeError("The metadata smoke command could not read the file")
    observed = json.loads(answer["stdout"])
    if args.baseline:
        baseline = json.loads(args.baseline.read_text())
        differences = {key: {"before": baseline.get(key), "after": observed.get(key)}
                       for key in sorted(baseline.keys() | observed.keys())
                       if baseline.get(key) != observed.get(key)}
        print(json.dumps({"metadata": observed, "matches_baseline": not differences,
                          "differences": differences}, indent=2))
    else:
        print(json.dumps(observed, indent=2))


def delete(api, args):
    api.request("DELETE", f"/sandboxes/{args.sandbox}", headers={
        "X-Ascii-Confirm-Delete": args.sandbox})
    print(json.dumps({"sandbox_id": args.sandbox, "delete_requested": True}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="action", required=True)
    command = commands.add_parser("create")
    command.add_argument("--size", choices=["small", "default", "large", "xlarge"], required=True)
    command.add_argument("--share-file", type=pathlib.Path)
    command.add_argument("--template", action="store_true")
    command.add_argument("--trial", action="store_true")
    command = commands.add_parser("prepare")
    command.add_argument("sandbox")
    command.add_argument("archive", type=pathlib.Path)
    command = commands.add_parser("snapshot")
    command.add_argument("sandbox")
    command.add_argument("name")
    command.add_argument("--share-file", type=pathlib.Path, required=True)
    command = commands.add_parser("run")
    command.add_argument("sandbox")
    command.add_argument("registration", type=pathlib.Path)
    command = commands.add_parser("status")
    command.add_argument("sandbox")
    command.add_argument("--process", type=int)
    command = commands.add_parser("delete")
    command.add_argument("sandbox")
    command = commands.add_parser("metadata")
    command.add_argument("sandbox")
    command.add_argument("--file", default="usr/bin/ping")
    command.add_argument("--baseline", type=pathlib.Path)
    args = parser.parse_args()
    try:
        globals()[args.action](Api(), args)
    except urllib.error.HTTPError as error:
        print(f"Boat request failed with HTTP {error.code}. The mutation was not retried.",
              file=sys.stderr)
        return 1
    except (OSError, ValueError, KeyError, RuntimeError, TimeoutError):
        print("The operation failed. Check inputs and provider state before retrying.",
              file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
