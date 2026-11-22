#!/usr/bin/env python3
"""Install the trusted pre-action hook into the pinned runner source."""

import argparse
from pathlib import Path
import shutil


# Microsoft.DevTunnels.Connections 1.3.60 brings Microsoft.AspNetCore.WebUtilities
# 8.0.25, which GHSA-x33c-w48q-qrgc (CVE-2026-114149) affects. The runner's
# NuGet audit fails the build on it. A direct reference raises the transitive
# package to the first patched 8.0 version. The audit stays on.
PATCHED_PACKAGES = """
  <ItemGroup>
    <PackageReference Include="Microsoft.AspNetCore.WebUtilities" Version="8.0.34" />
  </ItemGroup>
"""


def pin_patched_packages(project):
    content = project.read_text(encoding="utf-8-sig")
    if PATCHED_PACKAGES in content:
        return
    if content.count("</Project>") != 1:
        raise RuntimeError("Pinned runner Worker project changed")
    project.write_text(content.replace("</Project>", PATCHED_PACKAGES + "</Project>"),
                       encoding="utf-8-sig")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    args = parser.parse_args()
    worker = args.source / "src/Runner.Worker"
    runner = worker / "JobRunner.cs"
    content = runner.read_text(encoding="utf-8-sig")
    marker = "            ArgUtil.NotNull(message.Steps, nameof(message.Steps));"
    content = content.replace("\n            await CiproxyAssignment.PrepareAsync(message);", "")
    content = content.replace("\n            await CiproxyAssignment.PrepareAsync(message, jobRequestCancellationToken);", "")
    hook = """
            try
            {
                await CiproxyAssignment.PrepareAsync(message, jobRequestCancellationToken);
            }
            catch (OperationCanceledException) when (jobRequestCancellationToken.IsCancellationRequested)
            {
                return TaskResult.Canceled;
            }"""
    if hook not in content:
        if content.count(marker) != 1:
            raise RuntimeError("Pinned runner job boundary changed")
        content = content.replace(marker, marker + hook)
        runner.write_text(content, encoding="utf-8-sig")
    shutil.copy(Path(__file__).with_name("CiproxyAssignment.cs"), worker)
    pin_patched_packages(worker / "Runner.Worker.csproj")
    program = worker / "Program.cs"
    content = program.read_text(encoding="utf-8-sig")
    check = '''            if (args.Length == 1 && args[0] == "ciproxy-handoff-protocol-2") return 0;
'''
    if check not in content:
        marker = '            using (HostContext context = new("Worker"))'
        if content.count(marker) != 1:
            raise RuntimeError("Pinned runner startup boundary changed")
        program.write_text(content.replace(marker, check + marker), encoding="utf-8-sig")


if __name__ == "__main__":
    main()
