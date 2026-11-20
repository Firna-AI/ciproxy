# SPDX-License-Identifier: Apache-2.0
"""Contain relative installer files and execute finalization safely."""

import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile


def check_workdir(case, workspace):
    namespace = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    build = namespace / "opt/ciproxy-build"
    build.mkdir(parents=True)
    outside = namespace / "provider"
    outside.mkdir()
    (outside / "keep.txt").write_text("provider data\n")
    bookkeeping = namespace / "etc/ciproxy/provider-docker-gid"
    bookkeeping.parent.mkdir(parents=True)
    bookkeeping.write_text("988\n")
    group = namespace / "etc/group"
    group.write_text("docker:x:988:user\n")
    installer = build / "installer.sh"
    installer.write_text('printf "download\\n" > relative-download.deb\n'
                         'mkdir -p nested\ncd nested\nprintf "explicit cd\\n" > explicit.txt\n')
    (build / "plan.json").write_text(json.dumps({"steps": [{"id": "001-relative", "provisioner": 1,
        "type": "shell", "environment": {}, "script": "/opt/ciproxy-build/installer.sh",
        "interpreter": "bash", "user": "root"}]}))
    definition = pathlib.Path(__file__).resolve().parents[1]
    result = subprocess.run(["python3", str(definition / "boat-build-execute.py"), str(build), "--no-inventory"],
                            cwd=outside, capture_output=True, text=True)
    case.assertEqual(result.returncode, 0, result.stdout + result.stderr)
    case.assertTrue((build / "tmp/relative-download.deb").is_file(),
                    "Relative installer output must be inside the owned build directory.")
    case.assertFalse((outside / "relative-download.deb").exists())
    case.assertEqual((build / "tmp/nested/explicit.txt").read_text(), "explicit cd\n")
    finalize_fixture(case, namespace)
    case.assertFalse(build.exists(), "Finalization must remove owned installer scratch files.")
    case.assertFalse(bookkeeping.exists())
    case.assertEqual(group.read_text(), "docker:x:988:user\n")
    case.assertEqual((outside / "keep.txt").read_text(), "provider data\n")


def finalize_fixture(case, namespace):
    definition = pathlib.Path(__file__).resolve().parents[1]
    tools = namespace / "finalize-tools"
    tools.mkdir()
    python = tools / "python3"
    python.write_text("#!/bin/sh\n# This fixture supplies the passed-build prerequisite.\nexit 0\n")
    python.chmod(0o755)
    real_rm = shutil.which("rm")
    rm = tools / "rm"
    rm.write_text(f"#!{sys.executable}\nimport pathlib,subprocess,sys\n"
                  f"root=pathlib.Path({str(namespace)!r})\narguments=[]\n"
                  "for arg in sys.argv[1:]:\n"
                  "    if arg.startswith('/') and not arg.startswith(str(root)+'/'):\n"
                  "        arg=str(root/arg.lstrip('/'))\n"
                  "    arguments.append(arg)\n"
                  f"raise SystemExit(subprocess.run([{real_rm!r},*arguments]).returncode)\n")
    rm.chmod(0o755)
    finalizer = namespace / "finalize.sh"
    shutil.copy2(definition / "boat-build-finalize.sh", finalizer)
    result = subprocess.run(["bash", str(finalizer)], env={**os.environ,
        "PATH": str(tools) + os.pathsep + os.environ["PATH"]}, capture_output=True, text=True)
    case.assertEqual(result.returncode, 0, result.stdout + result.stderr)
