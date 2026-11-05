#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Run the generated QEMU bootstrap against offline Ubuntu source files."""

import argparse
import json
import os
import pathlib
import re
import shlex
import subprocess
import tempfile
import time
import unittest


def check_sources(workspace):
    workspace.mkdir(parents=True, exist_ok=True)
    root = pathlib.Path(tempfile.mkdtemp(prefix="template-check-", dir=workspace)).resolve()
    templates = root / "upstream/images/ubuntu/templates"
    templates.mkdir(parents=True)
    (templates / "build.ubuntu-24_04.pkr.hcl").write_text('''build {
  sources = ["source.azure-arm.image"]
  name = "ubuntu-24_04"
  provisioner "shell" {
    execute_command = "sudo sh -c '{{ .Vars }} {{ .Path }}'"
    inline          = ["sleep 30", "/usr/sbin/waagent -force -deprovision+user && export HISTSIZE=0 && sync"]
  }
}
''')
    (templates / "variable.ubuntu.pkr.hcl").touch()
    subprocess.run(["python3", str(pathlib.Path(__file__).with_name("prepare-template.py")),
                    str(root / "upstream"), str(root / "packer")], check=True)
    recipe = (root / "packer/build.pkr.hcl").read_text()
    inline = re.search(r'provisioner "shell"\s*\{\s*inline\s*=\s*', recipe)
    commands, _ = json.JSONDecoder().raw_decode(recipe[inline.end():])
    sources = root / "ubuntu.sources"
    sources.write_text('''# Ubuntu sources after cloud-init
Types: deb
URIs: http://archive.ubuntu.com/ubuntu
Suites: noble noble-updates noble-backports

Types: deb
URIs: http://security.ubuntu.com/ubuntu
Suites: noble-security
''')
    tools = root / "tools"
    tools.mkdir()
    for name, body in [("sudo", 'exec "$@"'), ("cloud-init", "exit 0"),
                       ("apt-get", "exit 0")]:
        tool = tools / name
        tool.write_text("#!/bin/sh\n" + body + "\n")
        tool.chmod(0o755)
    environment = {**os.environ, "LC_ALL": "C",
                   "PATH": str(tools) + os.pathsep + os.environ["PATH"]}
    script = "\n".join(command.replace("/etc/apt/sources.list.d/ubuntu.sources",
                                       shlex.quote(str(sources))) for command in commands)
    result = subprocess.run(["bash", "-e", "-c", script], env=environment,
                            capture_output=True, text=True)
    (root / "bootstrap.log").write_text(result.stdout + result.stderr)
    assert result.returncode == 0, "The generated bootstrap failed: " + result.stderr
    assert sources.read_text() == '''# Ubuntu sources after cloud-init
Types: deb
URIs: http://azure.archive.ubuntu.com/ubuntu/
Suites: noble noble-updates noble-backports

Types: deb
URIs: http://security.ubuntu.com/ubuntu
Suites: noble-security
''', "The generated bootstrap must prepare the primary URI for the upstream mirror-list check"
    print("Fixture evidence: " + str(root), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=pathlib.Path,
                        default=pathlib.Path(".context/ciproxy-mvp/kvm/template-check"))
    args = parser.parse_args()
    started = time.perf_counter()
    case = unittest.FunctionTestCase(lambda: check_sources(args.work_dir),
                                     description="generic Ubuntu sources enter the upstream mirror-list path")
    result = unittest.TextTestRunner(verbosity=2).run(unittest.TestSuite([case]))
    print(f"Regression command duration: {time.perf_counter() - started:.3f}s")
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
