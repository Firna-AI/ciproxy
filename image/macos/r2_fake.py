# SPDX-License-Identifier: Apache-2.0
"""A fake `aws` and a test base for check-r2.py.

The fake records each call and keeps the bucket in a folder. Environment
knobs set its version, a HEAD error, the size that HEAD reports, and a
manifest of another publisher.
"""

import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

ACCOUNT = "785233830ca9dfa2eb7c3561dd52f675"
KEY_ID = "fake-key-id-4f1c9a"
SECRET = "fake-secret-7d2e6b0c"

FAKE_AWS = r'''
import json, os, pathlib, shutil, sys
argv = sys.argv[1:]
if argv == ["--version"]:
    print(f"aws-cli/{os.environ.get('FAKE_AWS_VERSION', '2.31.0')} Python/3.13.7 Darwin/25.0.0")
    sys.exit(0)
config = os.environ.get("AWS_CONFIG_FILE", "")
env = {name: os.environ.get(name) for name in (
    "AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY", "AWS_PROFILE", "AWS_SESSION_TOKEN",
    "AWS_REQUEST_CHECKSUM_CALCULATION", "AWS_RESPONSE_CHECKSUM_VALIDATION",
    "R2_PUBLISH_ACCESS_KEY_ID", "R2_PUBLISH_SECRET_ACCESS_KEY", "R2_XCODE_ACCESS_KEY_ID",
    "R2_XCODE_SECRET_ACCESS_KEY")}
with open(os.environ["FAKE_AWS_LOG"], "a") as log:
    log.write(json.dumps({"argv": argv, "env": env, "config_file": config,
                          "config": pathlib.Path(config).read_text()}) + "\n")
root = pathlib.Path(os.environ["FAKE_R2"])
args = argv[4:]
value = lambda flag: args[args.index(flag) + 1]
def path(url):
    return root / url.removeprefix("s3://")
def fail(message):
    print(message, file=sys.stderr)
    sys.exit(254)
if args[:2] == ["s3api", "head-object"]:
    if os.environ.get("FAKE_AWS_HEAD_ERROR"):
        fail(os.environ["FAKE_AWS_HEAD_ERROR"])
    target = root / value("--bucket") / value("--key")
    if not target.exists():
        fail("An error occurred (404) when calling the HeadObject operation: Not Found")
    size = os.environ.get("FAKE_AWS_SIZE") or target.stat().st_size
    print(size if "--query" in args else "{}")
elif args[:2] == ["s3api", "put-object"]:
    target = root / value("--bucket") / value("--key")
    if "--if-none-match" in args and value("--if-none-match") == "*" and target.exists():
        fail("An error occurred (PreconditionFailed) when calling the PutObject operation")
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(value("--body"), target)
elif args[:2] == ["s3", "cp"]:
    source, destination = args[2:4]
    other = os.environ.pop("FAKE_AWS_OTHER_MANIFEST", "")
    if other and not (root / other).exists():
        (root / other).parent.mkdir(parents=True, exist_ok=True)
        (root / other).write_text("other publisher")
    source = path(source) if source.startswith("s3://") else pathlib.Path(source)
    destination = path(destination) if destination.startswith("s3://") else pathlib.Path(destination)
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)
else:
    fail(f"fake aws: unknown call {argv}")
'''

# macOS cp clones with -c. GNU cp has no -c, so the stub drops it.
FAKE_CP = '''#!/bin/sh
for argument; do
    shift
    [ "$argument" = -c ] || set -- "$@" "$argument"
done
exec {real} "$@"
'''


def sha256(data):
    return hashlib.sha256(data).hexdigest()


class FakeR2(unittest.TestCase):
    def setUp(self):
        self.root = pathlib.Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.root)
        self.bucket = self.root / "r2"
        self.log = self.root / "aws.log"
        tools = self.root / "bin"
        tools.mkdir()
        stubs = {"aws": f"#!{sys.executable} -I\n{FAKE_AWS}",
                 "curl": "#!/bin/sh\necho 'fake curl: no network' >&2\nexit 1\n",
                 "cp": FAKE_CP.format(real=shutil.which("cp"))}
        for name, text in stubs.items():
            (tools / name).write_text(text)
            (tools / name).chmod(0o755)
        self.env = {"PATH": f"{tools}:{os.environ['PATH']}", "HOME": str(self.root),
                    "TMPDIR": str(self.root), "FAKE_AWS_LOG": str(self.log),
                    "FAKE_R2": str(self.bucket), "AWS_PROFILE": "personal",
                    "AWS_SESSION_TOKEN": "user-session", "R2_ACCOUNT_ID": ACCOUNT}

    def run_script(self, *command, keys=(), env=None, trace=False):
        """Runs a script with the named key variables set. trace runs bash -x."""
        full_env = {**self.env, **(env or {})}
        for name in keys:
            full_env[name] = SECRET if "SECRET" in name else KEY_ID
        bash = ["bash", "-x"] if trace else ["bash"]
        return subprocess.run([*bash, *map(str, command)], env=full_env,
                              capture_output=True, text=True)

    def calls(self):
        if not self.log.exists():
            return []
        return [json.loads(line) for line in self.log.read_text().splitlines()]

    def assert_no_key(self, result):
        """The keys reach aws only in its environment, never the output or argv."""
        arguments = [" ".join(call["argv"]) for call in self.calls()]
        for text in (result.stdout, result.stderr, *arguments):
            self.assertNotIn(SECRET, text)
            self.assertNotIn(KEY_ID, text)

    def assert_r2_call(self, call, key_id, secret):
        self.assertEqual(call["argv"][:4], ["--endpoint-url",
                                            f"https://{ACCOUNT}.r2.cloudflarestorage.com",
                                            "--region", "auto"])
        self.assertEqual(call["env"], {
            "AWS_ACCESS_KEY_ID": key_id, "AWS_SECRET_ACCESS_KEY": secret,
            "AWS_PROFILE": None, "AWS_SESSION_TOKEN": None,
            "AWS_REQUEST_CHECKSUM_CALCULATION": "when_required",
            "AWS_RESPONSE_CHECKSUM_VALIDATION": "when_required",
            # Only r2_aws hands the keys on, under the AWS names.
            "R2_PUBLISH_ACCESS_KEY_ID": None, "R2_PUBLISH_SECRET_ACCESS_KEY": None,
            "R2_XCODE_ACCESS_KEY_ID": None, "R2_XCODE_SECRET_ACCESS_KEY": None})
        self.assertIn("multipart_chunksize = 134217728", call["config"])
        self.assertFalse(pathlib.Path(call["config_file"]).exists())
