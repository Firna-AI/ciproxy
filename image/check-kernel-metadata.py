#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check the kernel producer environment without compiling Linux."""

import argparse
import json
import os
import pathlib
import shutil
import subprocess
import tempfile
import time
import unittest


def check_metadata(workspace):
    workspace.mkdir(parents=True, exist_ok=True)
    root = pathlib.Path(tempfile.mkdtemp(prefix="kernel-metadata-", dir=workspace)).resolve()
    repository = root / "repository"
    image = repository / "image"
    image.mkdir(parents=True)
    shutil.copyfile(pathlib.Path(__file__).with_name("build-kernel.sh"), image / "build-kernel.sh")
    (image / "pins.env").write_text("KERNEL_VERSION=fixture\n")
    environment = {**os.environ, "LC_ALL": "C", "GIT_AUTHOR_DATE": "2000-01-01T00:00:00Z",
                   "GIT_COMMITTER_DATE": "2000-01-01T00:00:00Z"}
    for args in [["init", "-q"], ["add", "image/pins.env"],
                 ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                  "commit", "-qm", "Pin the kernel inputs"]]:
        subprocess.run(["git", "-C", str(repository), *args], env=environment, check=True)
    (repository / "notes").write_text("An unrelated source change\n")
    environment.update(GIT_AUTHOR_DATE="2000-01-02T00:00:00Z",
                       GIT_COMMITTER_DATE="2000-01-02T00:00:00Z")
    subprocess.run(["git", "-C", str(repository), "add", "notes"], check=True)
    subprocess.run(["git", "-C", str(repository), "-c", "user.name=Fixture", "-c",
                    "user.email=fixture@example.invalid", "commit", "-qm", "Update notes"],
                   env=environment, check=True)
    tools = root / "tools"
    tools.mkdir()
    make = tools / "make"
    make.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
names = ['KBUILD_BUILD_USER', 'KBUILD_BUILD_HOST', 'KBUILD_BUILD_TIMESTAMP',
         'KBUILD_BUILD_VERSION', 'SOURCE_DATE_EPOCH']
data = {name: os.environ.get(name) for name in names}
with open(os.environ['FIXTURE_CALLS'], 'a') as output:
    output.write(json.dumps(data) + '\\n')
if sys.argv[-1] == 'vmlinux':
    (pathlib.Path(sys.argv[2]) / 'vmlinux').write_text(json.dumps(data))
''')
    make.chmod(0o755)
    work = root / "work"
    (work / "linux-fixture").mkdir(parents=True)
    (work / "kernel.config").write_text("fixture config\n")
    environment.update(PATH=str(tools) + os.pathsep + os.environ["PATH"],
                       FIXTURE_CALLS=str(root / "make-calls.jsonl"),
                       KBUILD_BUILD_USER="private-account", KBUILD_BUILD_HOST="private-host",
                       KBUILD_BUILD_TIMESTAMP="wall clock", KBUILD_BUILD_VERSION="99")
    environment.pop("SOURCE_DATE_EPOCH", None)
    result = subprocess.run(["bash", str(image / "build-kernel.sh"), str(work)],
                            env=environment, capture_output=True, text=True)
    (root / "build.log").write_text(result.stdout + result.stderr)
    assert result.returncode == 0, result.stderr
    calls = [json.loads(line) for line in (root / "make-calls.jsonl").read_text().splitlines()]
    expected = {"KBUILD_BUILD_USER": "builder", "KBUILD_BUILD_HOST": "localhost",
                "KBUILD_BUILD_TIMESTAMP": "Sun Jan  2 00:00:00 UTC 2000",
                "KBUILD_BUILD_VERSION": "1", "SOURCE_DATE_EPOCH": "946771200"}
    assert calls == [expected, expected], "Both make calls must receive neutral, source-derived metadata"
    print("Fixture evidence: " + str(root), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=pathlib.Path,
                        default=pathlib.Path(".context/ciproxy-mvp/kvm/kernel-metadata"))
    args = parser.parse_args()
    started = time.perf_counter()
    case = unittest.FunctionTestCase(lambda: check_metadata(args.work_dir),
                                     description="kernel metadata excludes the build account and host")
    result = unittest.TextTestRunner(verbosity=2).run(unittest.TestSuite([case]))
    print(f"Regression command duration: {time.perf_counter() - started:.3f}s")
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
