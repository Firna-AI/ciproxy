# SPDX-License-Identifier: Apache-2.0
"""Check concrete package installation with the original logical CLI name."""

import json
import os
import pathlib
import subprocess
import tempfile

from pin_packages import apt_install_commands
from pin_toolset import apply_toolset


def check_alias(case, workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    toolset = root / "images/ubuntu/toolsets/toolset-2404.json"
    toolset.parent.mkdir(parents=True)
    toolset.write_text(json.dumps({"toolcache": [], "java": {}, "pwsh": {}, "node": {},
        "selenium": {}, "powershellModules": [], "docker": {"components": [], "plugins": []},
        "apt": {"cmd_packages": ["netcat", "zip"]}}))
    definition = pathlib.Path(__file__).resolve().parents[1]
    apply_toolset(root, json.loads((definition / "software-pins.json").read_text()))
    case.assertEqual(json.loads(toolset.read_text())["apt"]["cmd_packages"], ["netcat", "zip"])
    tools = root / "tools"
    tools.mkdir()
    apt = tools / "apt-get"
    apt.write_text("#!/usr/bin/env python3\nimport json,os,sys\n"
                   "with open(os.environ['APT_EVENTS'],'a') as f: f.write(json.dumps(sys.argv[1:])+'\\n')\n")
    apt.chmod(0o755)
    for name in ("netcat", "zip"):
        tool = tools / name
        tool.write_text("#!/bin/sh\nexit 0\n")
        tool.chmod(0o755)
    script = apt_install_commands('''for package in $PACKAGES; do
    apt-get install --no-install-recommends $package
done
for package in $PACKAGES; do command -v "$package" >/dev/null; done
''')
    events = root / "events.jsonl"
    environment = {**os.environ, "PATH": str(tools) + os.pathsep + os.environ["PATH"],
                   "PACKAGES": " ".join(json.loads(toolset.read_text())["apt"]["cmd_packages"]),
                   "APT_EVENTS": str(events)}
    result = subprocess.run(["bash", "-e", "-c", script], env=environment, capture_output=True, text=True)
    case.assertEqual(result.returncode, 0, result.stdout + result.stderr)
    case.assertEqual([json.loads(line) for line in events.read_text().splitlines()], [
        ["install", "--no-install-recommends", "netcat-openbsd"],
        ["install", "--no-install-recommends", "zip"]])
