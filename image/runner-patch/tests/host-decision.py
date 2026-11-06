#!/usr/bin/env python3
"""Run opt-in source-hook regressions against a built pinned Worker."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys
import uuid
import xml.etree.ElementTree as ET


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--worker-output", type=Path, required=True)
    parser.add_argument("--runner-bin", type=Path, required=True)
    parser.add_argument("--dotnet", default="dotnet")
    parser.add_argument("--work-dir", type=Path)
    parser.add_argument(
        "--case",
        choices=["all", "early_eof", "connect_error", "github_cancellation"],
        default="all",
    )
    args = parser.parse_args()
    repository = Path(__file__).resolve().parents[3]
    work = (args.work_dir or repository / ".context" / "host-decision-proof").resolve()
    work.mkdir(parents=True, exist_ok=True)
    worker = args.worker_output.resolve()
    runtime = args.runner_bin.resolve()
    project = ET.Element("Project", Sdk="Microsoft.NET.Sdk")
    properties = ET.SubElement(project, "PropertyGroup")
    for key, value in {
        "OutputType": "Exe",
        "TargetFramework": "net8.0",
        "ImplicitUsings": "enable",
        "Nullable": "enable",
        "EnableDefaultCompileItems": "false",
    }.items():
        ET.SubElement(properties, key).text = value
    references = ET.SubElement(project, "ItemGroup")
    for name in ["Sdk", "Runner.Sdk", "Runner.Common", "Runner.Worker", "Newtonsoft.Json", "System.Diagnostics.DiagnosticSource"]:
        reference = ET.SubElement(references, "Reference", Include=name)
        ET.SubElement(reference, "HintPath").text = str((worker if name == "Runner.Worker" else runtime) / f"{name}.dll")
    ET.SubElement(
        references,
        "Compile",
        Include=str(Path(__file__).with_name("HostDecisionProof.cs")),
    )
    project_path = work / "HostDecisionProof.csproj"
    ET.ElementTree(project).write(project_path, encoding="unicode")
    with (work / "build.log").open("w") as log:
        result = subprocess.run(
            [args.dotnet, "build", str(project_path), "--configuration", "Release"],
            stdout=log,
            stderr=subprocess.STDOUT,
            check=False,
        )
    if result.returncode:
        print((work / "build.log").read_text(), file=sys.stderr)
        return result.returncode
    socket_dir = repository / ".context" / f"hd-{uuid.uuid4().hex[:8]}"
    socket_dir.mkdir(parents=True)
    environment = os.environ.copy()
    environment["CIPROXY_PROOF_SOCKET_DIR"] = str(socket_dir)
    cases = (
        ["early_eof", "connect_error", "github_cancellation"]
        if args.case == "all"
        else [args.case]
    )
    try:
        for case in cases:
            result = subprocess.run(
                [
                    args.dotnet,
                    str(work / "bin/Release/net8.0/HostDecisionProof.dll"),
                    case,
                ],
                env=environment,
                check=False,
            )
            if result.returncode:
                return result.returncode
    finally:
        shutil.rmtree(socket_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
