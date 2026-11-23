#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Install a Worker overlay into a fresh official runner and load it."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys
import xml.etree.ElementTree as ET


def run(command, **options):
    print("$ " + " ".join(str(part) for part in command), flush=True)
    return subprocess.run(command, check=False, **options).returncode


def build_hook(work, dotnet):
    project = ET.Element("Project", Sdk="Microsoft.NET.Sdk")
    properties = ET.SubElement(project, "PropertyGroup")
    for key, value in {"TargetFramework": "net8.0", "Nullable": "enable",
                       "EnableDefaultCompileItems": "false"}.items():
        ET.SubElement(properties, key).text = value
    items = ET.SubElement(project, "ItemGroup")
    ET.SubElement(items, "Compile", Include=str(Path(__file__).with_name("OverlayLoadHook.cs")))
    path = work / "OverlayLoadHook.csproj"
    ET.ElementTree(project).write(path, encoding="unicode")
    with (work / "build.log").open("w") as log:
        code = run([dotnet, "build", str(path), "--configuration", "Release"],
                   stdout=log, stderr=subprocess.STDOUT)
    if code:
        print((work / "build.log").read_text(), file=sys.stderr)
        raise SystemExit(code)
    return work / "bin/Release/net8.0/OverlayLoadHook.dll"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runner-archive", type=Path, required=True)
    parser.add_argument("--overlay", type=Path, required=True)
    parser.add_argument("--dotnet", default="dotnet")
    parser.add_argument("--work-dir", type=Path)
    args = parser.parse_args()
    repository = Path(__file__).resolve().parents[3]
    work = (args.work_dir or repository / ".context" / "overlay-load").resolve()
    runner = work / "runner"
    shutil.rmtree(runner, ignore_errors=True)
    runner.mkdir(parents=True)
    if run(["tar", "-xzf", str(args.runner_archive), "-C", str(runner)]):
        raise SystemExit("FAIL: the runner archive did not extract")
    install = Path(__file__).resolve().parents[1] / "install-worker.sh"
    if run(["bash", str(install), str(runner), str(args.overlay.resolve())]):
        raise SystemExit("FAIL: install-worker.sh")
    hook = build_hook(work, args.dotnet)
    packages = (args.overlay / "packages.txt").read_text().strip().replace("\n", ";")
    worker = runner / "bin/Runner.Worker"
    for kind in ["stock", "own"]:
        shutil.copyfile(runner / f"bin/Runner.Worker.{kind}.dll", runner / "bin/Runner.Worker.dll")
        environment = {**os.environ, "DOTNET_STARTUP_HOOKS": str(hook),
                       "CIPROXY_OVERLAY_PACKAGES": packages}
        if run([str(worker)], env=environment):
            raise SystemExit(f"FAIL: the {kind} Worker did not load the overlay packages")
        print(f"PASS: the {kind} Worker loads the overlay packages")
    if run([str(worker), "ciproxy-handoff-protocol-2"]):
        raise SystemExit("FAIL: the own Worker format probe")
    print("PASS: the own Worker starts with the overlay deps file")
    if run([str(runner / "bin/Runner.Listener"), "--version"]):
        raise SystemExit("FAIL: Runner.Listener --version")
    print("PASS: the listener starts")


if __name__ == "__main__":
    main()
