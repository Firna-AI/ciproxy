# SPDX-License-Identifier: Apache-2.0
"""Resolve the pinned Ansible bundle against isolated local wheel metadata."""

import json
import os
import pathlib
import subprocess
import sys
import tempfile
import zipfile
from email.parser import Parser

from pin_release_tools import ansible_injection


def wheel(directory, name, version, requirement=None):
    distribution = name.replace("-", "_")
    metadata = f"Metadata-Version: 2.1\nName: {name}\nVersion: {version}\n"
    if requirement:
        metadata += "Requires-Dist: " + requirement + "\n"
    prefix = distribution + "-" + version + ".dist-info/"
    with zipfile.ZipFile(directory / (distribution + "-" + version + "-py3-none-any.whl"), "w") as output:
        output.writestr(prefix + "METADATA", metadata)
        output.writestr(prefix + "WHEEL", "Wheel-Version: 1.0\nRoot-Is-Purelib: true\nTag: py3-none-any\n")
        output.writestr(prefix + "RECORD", "")


def check_ansible(case, workspace):
    available = subprocess.run([sys.executable, "-m", "pip", "--version"], capture_output=True)
    if available.returncode:
        case.skipTest("pip is required for the Ansible dependency regression.")
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    packages, downloads, tools = root / "wheels", root / "downloads", root / "tools"
    for path in (packages, downloads, tools, root / "tmp"):
        path.mkdir()
    wheel(packages, "ansible-core", "2.21.6")
    wheel(packages, "ansible-core", "2.22.0")
    wheel(packages, "ansible", "14.6.0", "ansible-core~=2.21.6")
    wheel(packages, "ansible", "15.0.0", "ansible-core~=2.22.0")
    pipx = tools / "pipx"
    pipx.write_text(f"#!{sys.executable}\nimport os,subprocess,sys\n"
        "raise SystemExit(subprocess.run([sys.executable,'-m','pip','--isolated','--disable-pip-version-check',"
        "'--no-cache-dir','download','--no-index','--find-links',os.environ['FIXTURE_WHEELS'],"
        "'--dest',os.environ['FIXTURE_DOWNLOADS'],sys.argv[-1]]).returncode)\n")
    pipx.chmod(0o755)
    definition = pathlib.Path(__file__).resolve().parents[1]
    pins = json.loads((definition / "software-pins.json").read_text())
    script = ansible_injection("package=ansible-core\npipx inject $package ansible\n", pins["extra"]["ansible"])
    result = subprocess.run(["bash", "-e", "-c", script], env={**os.environ,
        "PATH": str(tools) + os.pathsep + os.environ["PATH"], "TMPDIR": str(root / "tmp"),
        "FIXTURE_WHEELS": str(packages), "FIXTURE_DOWNLOADS": str(downloads)}, capture_output=True, text=True)
    case.assertEqual(result.returncode, 0, result.stdout + result.stderr)
    versions = {}
    for path in downloads.glob("*.whl"):
        with zipfile.ZipFile(path) as archive:
            name = next(name for name in archive.namelist() if name.endswith("/METADATA"))
            metadata = Parser().parsestr(archive.read(name).decode())
            versions[metadata["Name"]] = metadata["Version"]
    case.assertEqual(versions["ansible-core"], pins["versions"]["Ansible"])
