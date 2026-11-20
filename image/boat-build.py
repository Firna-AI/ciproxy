#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Prepare, resume or report a native Boat image build without registration."""

import argparse
import json
import pathlib
import shlex

from boat import Api, prepare
from boat_build_bundle import bundle, checksum


def remote(api, args):
    if args.action == "prepare":
        return prepare(api, args)
    if args.action == "refresh":
        digest = checksum(args.archive)
        api.upload(args.sandbox, args.archive, "/tmp/ciproxy-native-refresh.tar.gz")
        api.upload(args.sandbox, pathlib.Path(__file__).with_name("boat-build-refresh.sh"),
                   "/tmp/ciproxy-refresh.sh")
        result = api.command(args.sandbox, "sudo bash /tmp/ciproxy-refresh.sh "
                             "/tmp/ciproxy-native-refresh.tar.gz " + digest)
        if result["exitCode"] != 0:
            raise RuntimeError("Native refresh did not start: " + result.get("stderr", ""))
        print(result["stdout"], end="")
        return
    if args.action == "finalize":
        args.action, args.log, args.logs = "report", None, True
        report = remote(api, args)
        if report.get("unavailable_artifacts"):
            raise RuntimeError("Keep build files until all required reports have been downloaded.")
        api.upload(args.sandbox, pathlib.Path(__file__).with_name("boat-build-finalize.sh"),
                   "/tmp/ciproxy-finalize.sh")
        result = api.command(args.sandbox, "sudo bash /tmp/ciproxy-finalize.sh")
        if result["exitCode"] != 0:
            raise RuntimeError("Native finalization failed: " + result.get("stderr", ""))
        print(result["stdout"], end="")
        return
    if args.action == "resume":
        command = ("sudo systemd-run --unit=ciproxy-image-build --collect /usr/bin/python3 "
                   "/opt/ciproxy-build/definition/boat-build-execute.py /opt/ciproxy-build --resume")
    elif args.log:
        command = "sudo cat " + shlex.quote("/opt/ciproxy-build/logs/" + args.log + ".log")
    else:
        command = "sudo cat /opt/ciproxy-build/report.json"
    answer = api.command(args.sandbox, command)
    if answer["exitCode"] != 0:
        raise RuntimeError("Native build command failed: " + answer.get("stderr", ""))
    if args.action == "report" and not args.log:
        report = json.loads(answer["stdout"])
        if args.output:
            args.output.mkdir(parents=True, exist_ok=True)
            for filename in ("installed-inventory.json", "installed-packages.json", "software-report.json",
                             "inventory-differences.json", "changed-services.json"):
                observed = api.command(args.sandbox, "sudo cat " +
                                       shlex.quote("/opt/ciproxy-build/reports/" + filename))
                if observed["exitCode"] == 0:
                    (args.output / filename).write_text(observed["stdout"])
                else:
                    report.setdefault("unavailable_artifacts", []).append(filename)
            if getattr(args, "logs", False):
                for provisioner in report["provisioners"]:
                    if "log" in provisioner:
                        path = provisioner["log"]
                        observed = api.command(args.sandbox, "sudo cat " +
                                               shlex.quote("/opt/ciproxy-build/" + path))
                        if observed["exitCode"] == 0:
                            destination = args.output / path
                            destination.parent.mkdir(parents=True, exist_ok=True)
                            destination.write_text(observed["stdout"])
                        else:
                            report.setdefault("unavailable_artifacts", []).append(path)
            (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report, indent=2))
        return report
    else:
        print(answer["stdout"], end="")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="action", required=True)
    command = commands.add_parser("bundle")
    for name in ("upstream-archive", "guest", "cache", "worker", "runner", "reference", "output"):
        command.add_argument("--" + name, type=pathlib.Path, required=True)
    command.add_argument("--metadata-cache", type=pathlib.Path)
    command = commands.add_parser("prepare")
    command.add_argument("sandbox")
    command.add_argument("archive", type=pathlib.Path)
    command = commands.add_parser("resume")
    command.add_argument("sandbox")
    command = commands.add_parser("refresh")
    command.add_argument("sandbox")
    command.add_argument("archive", type=pathlib.Path)
    command = commands.add_parser("report")
    command.add_argument("sandbox")
    command.add_argument("--log", help="Provisioner ID from report.json")
    command.add_argument("--output", type=pathlib.Path)
    command.add_argument("--logs", action="store_true")
    command = commands.add_parser("finalize")
    command.add_argument("sandbox")
    command.add_argument("--output", type=pathlib.Path, required=True)
    args = parser.parse_args()
    if args.action == "bundle":
        bundle(args)
    else:
        remote(Api(), args)


if __name__ == "__main__":
    main()
