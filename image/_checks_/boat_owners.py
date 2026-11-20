# SPDX-License-Identifier: Apache-2.0
"""Check the native ownership baseline and the pinned Pester adaptation."""

import contextlib
import importlib.util
import io
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import types
from unittest.mock import patch

from boat_build_native import adapt
from boat_build_owners import RECORD_NAME
from boat_build_report import collect


PINNED_TEST = '''    # https://github.com/actions/runner-images/issues/14477
    It "<Directory> is owned by root" -TestCases @(
        @{ Directory = "/usr" }
        @{ Directory = "/etc" }
        @{ Directory = "/usr/local" }
        @{ Directory = "/usr/local/bin" }
    ) {
        $(stat -c "%U" $Directory) | Should -Be "root"
    }
'''
TEST_NAME = "<Directory> keeps its owner from before the build"
REASON = ("Boat's base gives /usr/local and /usr/local/bin to its provider account; "
          "the test checks that no build step changes the owner recorded before the build; "
          "the machine image keeps the upstream root assertion.")


def check_owners(case, workspace):
    executor_path = pathlib.Path(__file__).resolve().parents[1] / "boat-build-execute.py"
    spec = importlib.util.spec_from_file_location("owner_executor", executor_path)
    executor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(executor)
    expected = {directory: subprocess.check_output(["stat", "-c", "%U", directory], text=True).strip()
                for directory in ("/usr", "/etc", "/usr/local", "/usr/local/bin")}
    for inventory in (False, True):
        with case.subTest(inventory=inventory):
            root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
            reader = root / "read-owners.py"
            reader.write_text('import json,pathlib,sys\n'
                              'root=pathlib.Path(sys.argv[1])\n'
                              f'record=root/"{RECORD_NAME}"\n'
                              'if not record.is_file():\n'
                              '    print("Provider owners must exist before the first step.")\n'
                              '    raise SystemExit(23)\n'
                              'owners=json.loads(record.read_text())\n'
                              '(root/"owners-seen.json").write_text(json.dumps(owners))\n')
            step = {"id": "first", "provisioner": 0, "type": "shell", "environment": {},
                    "inline": [f'{sys.executable} "{reader}" /opt/ciproxy-build'], "inputs": [str(reader)]}
            (root / "plan.json").write_text(json.dumps({"steps": [step]}))

            def run(resume=False):
                argv = [str(executor_path), str(root)]
                argv += [] if inventory else ["--no-inventory"]
                argv += ["--resume"] if resume else []
                with patch.object(sys, "argv", argv), patch.dict(os.environ), \
                        patch.object(executor, "collect"), \
                        patch.object(executor, "service_inventory", return_value={}), \
                        contextlib.redirect_stdout(io.StringIO()):
                    code = executor.main()
                case.assertEqual(code, 0, (root / "logs/first.log").read_text())

            run()
            record = root / RECORD_NAME
            case.assertEqual(json.loads(record.read_text()), expected)
            case.assertEqual(json.loads((root / "owners-seen.json").read_text()), expected)
            earlier = {**expected, "/usr/local": "provider-before-build"}
            original = (json.dumps(earlier, indent=4) + "\n\n").encode()
            record.write_bytes(original)
            reader.write_text(reader.read_text() + "\n# Replay the fixture step.\n")
            run(True)
            case.assertEqual(record.read_bytes(), original)
            case.assertEqual(json.loads((root / "owners-seen.json").read_text()), earlier)


def native_fixture(workspace, block=PINNED_TEST):
    upstream = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    scripts = upstream / "images/ubuntu/scripts"
    files = {
        "build/configure-environment.sh": (
            "# Change waagent entries\n# Add localhost alias\n"
            "# Relax root filesystem durability\n# Create symlink for tests\n"
            "mkdir $AGENT_TOOLSDIRECTORY\n"
            "ln -s $HELPER_SCRIPTS/invoke-tests.sh /usr/local/bin/invoke_tests\n"),
        "build/configure-apt-sources.sh": "touch /etc/apt/apt-mirrors.txt\n",
        "build/configure-system.sh": "mv -f /imagegeneration/post-generation /opt\n",
        "build/install-docker.sh": 'groupmod -g "$gid" docker\n# Docker daemon takes time\n',
        "build/install-nodejs.sh": "",
        "build/install-pypy.sh": 'install_pypy() {\n    local package_url=$1\n    mkdir $pypy_toolcache_version_path\n}\n',
        "build/cleanup.sh": "",
        "helpers/install.sh": "",
        "tests/System.Tests.ps1": "",
        "../assets/post-gen/cleanup-logs.sh": "",
    }
    for name, content in files.items():
        path = scripts / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
    prefix = b'# Keep these bytes.\r\nDescribe "Containers" {\n    It "podman" { 1 | Should -Be 1 }\n\n'
    suffix = b'\n    It "fusermount3" { 1 | Should -Be 1 }\r\n}\nDescribe "other" {}\n'
    tools = scripts / "tests/Tools.Tests.ps1"
    tools.write_bytes(prefix + block.encode() + suffix)
    return upstream, tools, prefix, suffix


def check_adaptation(case, workspace):
    upstream, tools, prefix, suffix = native_fixture(workspace)
    before = tools.read_bytes()
    changes = adapt(upstream)
    after = tools.read_bytes()
    case.assertIn(f'It "{TEST_NAME}" -TestCases @('.encode(), after)
    replacement = after[len(prefix):-len(suffix)]
    case.assertEqual(prefix + replacement + suffix, after)
    case.assertEqual(after.replace(replacement, PINNED_TEST.encode(), 1), before)
    cases = PINNED_TEST.split(' -TestCases @(', 1)[1].split('    ) {', 1)[0]
    case.assertIn((' -TestCases @(' + cases + '    ) {').encode(), replacement)
    case.assertTrue(replacement.startswith(PINNED_TEST.split('    It ', 1)[0].encode()))
    case.assertIn(f'Get-Content "/opt/ciproxy-build/{RECORD_NAME}" -Raw -ErrorAction Stop'.encode(), replacement)
    case.assertIn(b'ConvertFrom-Json -AsHashtable -ErrorAction Stop', replacement)
    case.assertIn(b'if (-not $owners.ContainsKey($Directory)) {', replacement)
    case.assertIn(b'throw "No owner was recorded before the build for $Directory"', replacement)
    case.assertIn(b'$(stat -c "%U" $Directory) | Should -Be $owners[$Directory]', replacement)
    case.assertNotIn(b'Should -Be "root"', replacement)
    case.assertEqual([change for change in changes if change.get("test", "").startswith("Tools.Tests.ps1:")],
                     [{"test": "Tools.Tests.ps1: " + TEST_NAME, "reason": REASON}])


def check_changed_directories(case, workspace):
    upstream, tools, _, _ = native_fixture(workspace, PINNED_TEST.replace('"/usr/local/bin"', '"/opt"'))
    before = tools.read_bytes()
    with case.assertRaisesRegex(RuntimeError, "^Pinned native adaptation does not match Tools.Tests.ps1$"):
        adapt(upstream)
    case.assertEqual(tools.read_bytes(), before)


def check_inventory(case, workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    packages = types.SimpleNamespace(returncode=0, stdout="fixture-package\t1\n")
    record = root / RECORD_NAME
    inventory = root / "reports/installed-inventory.json"
    expected = {"/usr": "root", "/etc": "root", "/usr/local": "user", "/usr/local/bin": "user"}
    with patch("boat_build_report.subprocess.run", return_value=packages):
        collect(root, root / "imagegeneration", root / "restart-services")
        case.assertIn("provider_directory_owners", json.loads(inventory.read_text()))
        case.assertIsNone(json.loads(inventory.read_text())["provider_directory_owners"])
        record.write_text(json.dumps(expected))
        collect(root, root / "imagegeneration", root / "restart-services")
    case.assertEqual(json.loads(inventory.read_text())["provider_directory_owners"], expected)
