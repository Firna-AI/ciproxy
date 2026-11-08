"""The public launcher gives ordinary clients one private local endpoint."""

import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest

from deploy.tests.promotion_support import ROOT


PROGRAM = r'''
import json
import os
from pathlib import Path
import sys

tool, args = Path(sys.argv[0]).name, sys.argv[1:]
directory = Path(os.environ["LAUNCHER_FIXTURE"])
if tool == "sudo":
    args.remove("-n")
    if args[0] in ("test", "chown"):
        sys.exit(0)
    os.execvp(args[0], args)
elif tool == "buildkitd":
    (directory / "daemon.json").write_text(json.dumps({"args": args}))
    with (directory / "ready.fifo").open("w") as output:
        output.write("ready\n")
elif tool == "buildctl":
    marker = directory / "probe-started"
    if not marker.exists():
        marker.touch()
        with (directory / "ready.fifo").open() as ready:
            ready.readline()
    with (directory / "clients.jsonl").open("a") as output:
        output.write(json.dumps({"args": args, "host": os.environ.get("BUILDKIT_HOST", "")}) + "\n")
    if os.environ.get("LAUNCHER_FAILURE"):
        print("local socket: permission denied; fixture-private-error-payload", file=sys.stderr)
        sys.exit(1)
'''


class BuildkitLauncherTests(unittest.TestCase):
    def setUp(self):
        scratch = ROOT / ".context/control-plane-image/fix-round-2/tests"
        scratch.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        tools = self.root / "bin"
        tools.mkdir()
        for name in ("sudo", "buildkitd", "buildkit-runc", "buildctl", "sleep"):
            path = tools / name
            path.write_text(f"#!{sys.executable}\n" + PROGRAM)
            path.chmod(0o755)
        os.mkfifo(self.root / "ready.fifo")
        self.github_env = self.root / "github-env"
        self.github_env.touch()
        self.env = dict(os.environ, PATH=str(tools) + os.pathsep + os.environ["PATH"],
                        LAUNCHER_FIXTURE=str(self.root), GITHUB_ENV=str(self.github_env))
        self.env.pop("BUILDKIT_HOST", None)
        scripts = self.root / "deploy/scripts"
        scripts.mkdir(parents=True)
        shutil.copyfile(ROOT / "deploy/scripts/start-buildkit.sh", scripts / "start-buildkit.sh")

    def launch(self, failure=False):
        environment = dict(self.env, LAUNCHER_FAILURE="true") if failure else self.env
        result = subprocess.run(["bash", "deploy/scripts/start-buildkit.sh"],
                                cwd=self.root, env=environment, text=True, capture_output=True)
        calls = [json.loads(line) for line in (self.root / "clients.jsonl").read_text().splitlines()]
        host = calls[0]["host"]
        if host.startswith("unix:///tmp/ciproxy-bk."):
            self.addCleanup(shutil.rmtree, Path(host.removeprefix("unix://")).parent, True)
        return result, calls

    def test_launcher_shares_one_private_endpoint_with_readiness_and_future_clients(self):
        result, calls = self.launch()
        self.assertEqual(result.returncode, 0, result.stderr)
        host = calls[0]["host"]
        self.assertEqual(self.github_env.read_text(), "BUILDKIT_HOST=" + host + "\n")
        self.assertTrue(host.startswith("unix:///tmp/ciproxy-bk."), host)
        socket = Path(host.removeprefix("unix://"))
        self.assertLess(len(str(socket).encode()), 104)
        self.assertEqual(stat.S_IMODE(socket.parent.stat().st_mode), 0o700)
        self.assertEqual(socket.parent.stat().st_uid, os.getuid())
        arguments = json.loads((self.root / "daemon.json").read_text())["args"]
        self.assertEqual(arguments[arguments.index("--addr") + 1], host)
        self.assertEqual(arguments[arguments.index("--group") + 1], str(os.getgid()))
        self.assertEqual(arguments[arguments.index("--root") + 1], str(socket.parent / "state"))
        handoff = self.root / ".context/control-plane-image/buildkit-env.sh"
        self.assertEqual(stat.S_IMODE(handoff.stat().st_mode), 0o600)
        future = subprocess.run(["bash", "-c", '. .context/control-plane-image/buildkit-env.sh\nbuildctl debug workers'],
                                cwd=self.root, env=self.env, text=True, capture_output=True)
        self.assertEqual(future.returncode, 0, future.stderr)
        calls = [json.loads(line) for line in (self.root / "clients.jsonl").read_text().splitlines()]
        self.assertEqual(calls, [{"args": ["debug", "workers"], "host": host},
                                 {"args": ["debug", "workers"], "host": host}])

    def test_failed_startup_reports_the_local_cause_without_raw_payload(self):
        result, calls = self.launch(failure=True)
        self.assertEqual(result.returncode, 1)
        self.assertIn("permission denied", result.stderr)
        self.assertNotIn("fixture-private-error-payload", result.stdout + result.stderr)
        self.assertEqual(self.github_env.read_text(), "")
        self.assertTrue(all(call["host"] == calls[0]["host"] for call in calls))
