# SPDX-License-Identifier: Apache-2.0
"""Exercise rendered Java requests and non-failing APT pin reports."""

import json
import os
import pathlib
import subprocess
import sys
import tempfile
import types
from unittest.mock import patch

from boat_build_report import collect
from _checks_.apt_fixture import repository
from _checks_.recipe_fixture import rendered


def check_java_request(case, workspace):
    root, upstream, _ = rendered(workspace)
    scripts = upstream / "images/ubuntu/scripts/build"
    configured = (scripts / "configure-apt.sh").read_text()
    preferences = configured.split("<<'CIPROXY_PINS'\n", 1)[1].split("\nCIPROXY_PINS", 1)[0]
    packages = "\n".join(
        "Package: temurin-17-jdk\nVersion: " + version + "\nArchitecture: amd64\n"
        "Description: Temurin fixture\nFilename: java.deb\nSize: 1\n"
        for version in ("17.0.21.0.0+9-0", "17.0.22.0.0+7-0"))
    apt = repository(case, workspace, packages, preferences)
    update = apt.run(apt.apt_get, "update")
    case.assertEqual(update.returncode, 0, update.stderr)
    tools = root / "tools"
    tools.mkdir()
    solver = root / "solver.log"
    wrapper = tools / "apt-get"
    wrapper.write_text(f'#!{sys.executable}\nimport pathlib,subprocess,sys\n'
                       f'command={[apt.apt_get, *apt.configuration, "--simulate", "--allow-downgrades"]!r}\n'
                       'result=subprocess.run(command+sys.argv[1:],capture_output=True,text=True)\n'
                       f'pathlib.Path({str(solver)!r}).write_text(result.stdout+result.stderr)\n'
                       'raise SystemExit(result.returncode)\n')
    wrapper.chmod(0o755)
    home = root / "jvm/temurin-17-jdk-amd64"
    home.mkdir(parents=True)
    (home / "release").write_text('SEMANTIC_VERSION="17.0.21+9"\n')
    java = (scripts / "install-java-tools.sh").read_text()
    start = java.index("install_open_jdk() {")
    end = java.index("\n}\n", start) + 3
    script = java[start:end].replace("/usr/lib/jvm", str(root / "jvm")) + "\njava_arch=amd64\ninstall_open_jdk 17\n"
    result = subprocess.run(["bash", "-e", "-c", script], capture_output=True, text=True,
                            env={**os.environ, "PATH": str(tools) + os.pathsep + os.environ["PATH"],
                                 "AGENT_TOOLSDIRECTORY": str(root / "toolcache")})
    case.assertEqual(result.returncode, 0, result.stdout + result.stderr)
    case.assertIn("Inst temurin-17-jdk (17.0.21.0.0+9-0", solver.read_text())
    case.assertNotIn("Inst temurin-17-jdk (17.0.22", solver.read_text())


def check_pin_reports(case, workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    system, artifacts, tools = root / "system", root / "artifacts", root / "tools"
    for path in (system / "var/lib/dpkg", artifacts, tools):
        path.mkdir(parents=True)
    case.assertFalse((system / "etc/apt/preferences.d/ciproxy-software").exists())
    (system / "var/lib/dpkg/status").write_text(
        "Package: temurin-17-jdk\nStatus: install ok installed\nVersion: 17.0.22.0.0+7-0\n\n"
        "Package: libsystemd0\nSource: systemd\nStatus: install ok installed\nVersion: 255.4-1ubuntu8.19\n\n"
        "Package: other\nStatus: install ok installed\nVersion: 1\n")
    expected = {"available": True, "differences_fail_build": False, "difference_count": 2, "differences": [
        {"package": "libsystemd0", "installed": "255.4-1ubuntu8.19", "pin": "255.4-1ubuntu8.18"},
        {"package": "temurin-17-jdk", "installed": "17.0.22.0.0+7-0", "pin": "17.0.21.0.0+9-*"}]}
    (artifacts / "rootfs.ext4").write_bytes(b"fixture")
    (artifacts / "vmlinux").write_bytes(b"fixture")
    dumpe2fs = tools / "dumpe2fs"
    dumpe2fs.write_text("#!/bin/sh\nprintf 'Block count: 100\\nFree blocks: 20\\nReserved block count: 1\\nBlock size: 4096\\n'\n")
    dumpe2fs.chmod(0o755)
    image = pathlib.Path(__file__).resolve().parents[1]
    result = subprocess.run([sys.executable, str(image / "measure.py"), str(system), "--artifacts", str(artifacts)],
                            capture_output=True, text=True,
                            env={**os.environ, "PATH": str(tools) + os.pathsep + os.environ["PATH"]})
    case.assertEqual(result.returncode, 0, result.stderr)
    measurement = json.loads(result.stdout)
    with case.subTest(report="machine measurement without preferences"):
        case.assertEqual(measurement["apt_pin_differences"], expected)
    build, inventory = root / "build", root / "imagegeneration"
    build.mkdir()
    inventory.mkdir()
    (build / "reference-software.json").write_text("{}")
    (inventory / "software-report.json").write_text("{}")
    packages = types.SimpleNamespace(returncode=0, stdout="temurin-17-jdk\t17.0.22.0.0+7-0\n")
    with patch("boat_build_report.subprocess.run", return_value=packages):
        case.assertIsNone(collect(build, inventory, root / "restart", system_root=system))
    for filename in ("installed-inventory.json", "inventory-differences.json"):
        reported = json.loads((build / "reports" / filename).read_text())["apt_pin_differences"]
        with case.subTest(report="Boat " + filename + " without preferences"):
            case.assertEqual(reported, expected)
