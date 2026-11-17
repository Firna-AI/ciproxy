#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check the real guest launcher and runner entry without KVM or GitHub."""

import argparse
import json
import os
import pathlib
import re
import shutil
import subprocess
import tempfile
import time
import unittest


REGISTRATION = "fixture registration '$HOME; \"quoted\" $(false)"
ACCOUNT_PATH = "/srv/runner/bin:/opt/image-tools:/usr/bin:/bin"


def copy_program(source, root, destination=None):
    source = pathlib.Path(source).resolve()
    target = root / (destination or str(source)).lstrip("/")
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)
    target.chmod(0o755)
    if source.read_bytes().startswith(b"#!"):
        interpreter = source.read_text().splitlines()[0][2:].split()[0]
        copy_program(interpreter, root)
        return
    dependencies = subprocess.check_output(["ldd", str(source)], text=True)
    for library in re.findall(r"(/[^\s()]+)", dependencies):
        target = root / library.lstrip("/")
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(library, target)
        target.chmod(0o755)


def prepare_root(root, guest):
    for relative in ["etc/pam.d", "dev", "root", "srv/runner", "observations",
                     "opt/actions-runner/bin", "usr/local/libexec"]:
        (root / relative).mkdir(parents=True, exist_ok=True)
    for name in ["bash", "env", "cp"]:
        copy_program(shutil.which(name), root, "/usr/bin/" + name)
    (root / "bin").symlink_to("usr/bin")
    copy_program("/usr/sbin/runuser", root)
    copy_program(guest, root, "/usr/local/bin/ciproxy-guest")
    certificates = pathlib.Path("/etc/ssl/certs/ca-certificates.crt")
    for relative in ["etc/ssl/certs/ca-certificates.crt", "etc/pki/tls/certs/ca-bundle.crt"]:
        (root / relative).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(certificates, root / relative)
    modules = [str(module) for directory in [pathlib.Path("/usr/lib"), pathlib.Path("/usr/lib64")]
               for module in directory.glob("**/pam_permit.so")]
    assert modules, "The fixture needs the PAM permit module"
    copy_program(modules[0], root)
    (root / "etc/pam.d/runuser").write_text(
        f"auth required {modules[0]}\naccount required {modules[0]}\n"
        f"session required {modules[0]}\n")
    (root / "etc/passwd").write_text(
        "root:x:0:0:Root:/root:/bin/bash\n"
        "runner:x:12345:12345:Runner:/srv/runner:/bin/bash\n")
    (root / "etc/group").write_text("root:x:0:\nrunner:x:12345:\n")
    (root / "etc/nsswitch.conf").write_text("passwd: files\ngroup: files\n")
    (root / "dev/null").touch()
    (root / "dev/null").chmod(0o666)
    (root / "etc/profile").write_text("export PATH=/usr/bin:/bin\n")
    (root / "etc/environment").write_text(
        "PATH=/opt/image-tools:/usr/bin:/bin\nCIPROXY_IMAGE_ENV=image-tools\n")
    (root / "srv/runner/.profile").write_text(
        'export PATH="$HOME/bin:$PATH"\nexport CIPROXY_ACCOUNT_ENV=runner-profile\n')
    shutil.copyfile(pathlib.Path(__file__).with_name("runner-entry.sh"),
                    root / "usr/local/libexec/ciproxy-runner")
    (root / "usr/local/libexec/ciproxy-runner").chmod(0o755)
    runner = root / "opt/actions-runner"
    for name in ["Runner.Worker.stock.dll", "Runner.Worker.own.dll"]:
        (runner / "bin" / name).write_text(name + "\n")
    listener = runner / "bin/Runner.Listener"
    listener.write_text('''#!/usr/bin/bash
printf '%s\\0' "$HOME" "$USER" "$LOGNAME" "$SHELL" "$PATH" \\
    "$ACTIONS_RUNNER_INPUT_JITCONFIG" "${CIPROXY_ASSIGNMENT_SOCKET+x}" \\
    "${CIPROXY_ASSIGNMENT_SOCKET-}" "${CIPROXY_ROOT_ONLY_FIXTURE-}" \\
    "$CIPROXY_IMAGE_ENV" "$CIPROXY_ACCOUNT_ENV" > /observations/listener.env
printf 'fixture: Listening for Jobs\\n'
''')
    listener.chmod(0o755)
    os.chown(root / "observations", 12345, 12345)
    for path in [runner, *runner.rglob("*")]:
        os.chown(path, 12345, 12345)
    (root / "registration.json").write_text(json.dumps({
        "configure": {"type": "configure_runner", "protocol_version": 1,
                      "runner_id": "fixture-runner", "organization_id": "fixture-org",
                      "registration": REGISTRATION, "cache": None},
        "exit_report_url": "https://127.0.0.1:1/unused",
        "exit_report_token": "fixture-exit-token",
    }))
    (root / "registration.json").chmod(0o600)


def check_environment(workspace, guest):
    assert os.geteuid() == 0, "Run this fixture with sudo for an isolated account database"
    workspace.mkdir(parents=True, exist_ok=True)
    root = pathlib.Path(tempfile.mkdtemp(prefix="runner-environment-", dir=workspace)).resolve()
    root.chmod(0o755)
    prepare_root(root, guest)
    environment = {"LC_ALL": "C", "TZ": "UTC", "PATH": "/usr/bin:/bin:/root-only/bin",
                   "USER": "root", "LOGNAME": "root", "SHELL": "/bin/false",
                   "CIPROXY_ROOT_ONLY_FIXTURE": "must-not-reach-runner"}
    result = subprocess.run(
        [shutil.which("chroot"), str(root), "/usr/local/bin/ciproxy-guest",
         "--image-id", "fixture-image", "boat", "--registration-file", "/registration.json"],
        env=environment, capture_output=True, text=True, timeout=30)
    (root / "guest.log").write_text(result.stdout + result.stderr)
    print("Fixture evidence: " + str(root), flush=True)
    assert result.returncode == 0, f"Guest failed: {result.stdout}{result.stderr}"
    observation = root / "observations/listener.env"
    assert observation.exists(), "The guest must start the listener with its account environment"
    names = ["HOME", "USER", "LOGNAME", "SHELL", "PATH", "JIT", "SOCKET_SET", "SOCKET",
             "ROOT_ONLY", "IMAGE_ENV", "ACCOUNT_ENV"]
    actual = dict(zip(names, observation.read_text().split("\0")[:-1]))
    (root / "environment.json").write_text(json.dumps(actual, indent=2) + "\n")
    assert actual["HOME"] == "/srv/runner", "The runner needs HOME from its account"
    assert actual["USER"] == actual["LOGNAME"] == "runner", actual
    assert actual["SHELL"] == "/bin/bash", actual
    assert actual["PATH"] == ACCOUNT_PATH, "Keep image tools and the runner account PATH"
    assert actual["JIT"] == REGISTRATION, "Keep the registration without shell expansion"
    assert actual["SOCKET_SET"] == "x" and actual["SOCKET"] == "", actual
    assert actual["ROOT_ONLY"] == "", "Do not inherit the root service environment"
    assert actual["IMAGE_ENV"] == "image-tools", actual
    assert actual["ACCOUNT_ENV"] == "runner-profile", actual


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--guest", type=pathlib.Path,
                        default=pathlib.Path("target/debug/ciproxy-guest"))
    parser.add_argument("--work-dir", type=pathlib.Path,
                        default=pathlib.Path(".context/live-run-2026-11-16/runner"))
    args = parser.parse_args()
    started = time.perf_counter()
    case = unittest.FunctionTestCase(
        lambda: check_environment(args.work_dir, args.guest),
        description="runner account environment survives a root service with no HOME")
    result = unittest.TextTestRunner(verbosity=2).run(unittest.TestSuite([case]))
    print(f"Regression command duration: {time.perf_counter() - started:.3f}s")
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
