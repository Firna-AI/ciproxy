#!/usr/bin/env python3
"""Install the trusted pre-action hook into the pinned runner source."""

import argparse
from pathlib import Path
import shutil


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    args = parser.parse_args()
    worker = args.source / "src/Runner.Worker"
    runner = worker / "JobRunner.cs"
    content = runner.read_text(encoding="utf-8-sig")
    marker = "            ArgUtil.NotNull(message.Steps, nameof(message.Steps));"
    hook = "\n            await CiproxyAssignment.PrepareAsync(message);"
    if hook not in content:
        if content.count(marker) != 1:
            raise RuntimeError("Pinned runner job boundary changed")
        content = content.replace(marker, marker + hook)
        runner.write_text(content, encoding="utf-8-sig")
    shutil.copy(Path(__file__).with_name("CiproxyAssignment.cs"), worker)


if __name__ == "__main__":
    main()
