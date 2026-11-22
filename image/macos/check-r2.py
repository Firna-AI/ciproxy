#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check publish.sh and the Xcode download of fetch.sh offline.

A fake `aws` on PATH records each call and keeps the bucket in a folder.
Needs bash and shasum.
"""

import hashlib
import io
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile

HERE = pathlib.Path(__file__).resolve().parent
ACCOUNT = "785233830ca9dfa2eb7c3561dd52f675"
KEY_ID = "fake-key-id-4f1c9a"
SECRET = "fake-secret-7d2e6b0c"
IMAGE_ID = "macos26-arm64-check-v1"
PINS = dict(line.split("=", 1) for line in (HERE / "pins.env").read_text().splitlines()
            if line and not line.startswith("#"))

FAKE_AWS = r'''
import json, os, pathlib, shutil, sys
argv = sys.argv[1:]
if argv == ["--version"]:
    print("aws-cli/2.31.0 Python/3.13.7 Darwin/25.0.0 source/arm64")
    sys.exit(0)
config = os.environ.get("AWS_CONFIG_FILE", "")
env = {name: os.environ.get(name) for name in (
    "AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY", "AWS_PROFILE", "AWS_SESSION_TOKEN",
    "AWS_REQUEST_CHECKSUM_CALCULATION", "AWS_RESPONSE_CHECKSUM_VALIDATION")}
with open(os.environ["FAKE_AWS_LOG"], "a") as log:
    log.write(json.dumps({"argv": argv, "env": env, "config_file": config,
                          "config": pathlib.Path(config).read_text()}) + "\n")
root = pathlib.Path(os.environ["FAKE_R2"])
args = argv[4:]
value = lambda flag: args[args.index(flag) + 1]
def path(url):
    return root / url.removeprefix("s3://")
if args[:2] == ["s3api", "head-object"]:
    target = root / value("--bucket") / value("--key")
    if not target.exists():
        print("An error occurred (404) when calling the HeadObject operation: Not Found",
              file=sys.stderr)
        sys.exit(254)
    print(target.stat().st_size if "--query" in args else "{}")
elif args[:2] == ["s3api", "put-object"]:
    target = root / value("--bucket") / value("--key")
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(value("--body"), target)
elif args[:2] == ["s3", "cp"]:
    source, destination = args[2:4]
    source = path(source) if source.startswith("s3://") else pathlib.Path(source)
    destination = path(destination) if destination.startswith("s3://") else pathlib.Path(destination)
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)
else:
    sys.exit(f"fake aws: unknown call {argv}")
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
        (tools / "aws").write_text(f"#!{sys.executable} -I\n{FAKE_AWS}")
        (tools / "aws").chmod(0o755)
        (tools / "curl").write_text("#!/bin/sh\necho 'fake curl: no network' >&2\nexit 1\n")
        (tools / "curl").chmod(0o755)
        self.env = {"PATH": f"{tools}:{os.environ['PATH']}", "HOME": str(self.root),
                    "TMPDIR": str(self.root), "FAKE_AWS_LOG": str(self.log),
                    "FAKE_R2": str(self.bucket), "AWS_PROFILE": "personal",
                    "AWS_SESSION_TOKEN": "user-session", "R2_ACCOUNT_ID": ACCOUNT}

    def run_script(self, *command, keys=()):
        env = dict(self.env)
        for name in keys:
            env[name] = SECRET if "SECRET" in name else KEY_ID
        return subprocess.run(["bash", *map(str, command)], env=env, capture_output=True,
                              text=True)

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
            "AWS_RESPONSE_CHECKSUM_VALIDATION": "when_required"})
        self.assertIn("multipart_chunksize = 134217728", call["config"])
        self.assertFalse(pathlib.Path(call["config_file"]).exists())


PUBLISH_KEYS = ("R2_PUBLISH_ACCESS_KEY_ID", "R2_PUBLISH_SECRET_ACCESS_KEY")
SMALL_FILES = ["downloads.json", "SHA256SUMS", "measurement.json", "software-report.json",
               "upstream-software.json", "size-report.txt"]


class Publish(FakeR2):
    def artifacts(self, test_image=False, size_delta=0):
        folder = self.root / "artifacts"
        folder.mkdir()
        tar = b"macos archive" * 1000
        files = {"macos.tar": tar, "measurement.json": b"{}", "software-report.json": b"{}",
                 "upstream-software.json": b"{}", "size-report.txt": b"macos.tar: 13000\n",
                 "manifest.json": json.dumps({"image_id": IMAGE_ID,
                                              "test_image": test_image}).encode(),
                 "downloads.json": json.dumps({"macos": {
                     "file": "macos.tar", "sha256": sha256(tar),
                     "size_bytes": len(tar) + size_delta}}).encode()}
        for name, data in files.items():
            (folder / name).write_bytes(data)
        (folder / "SHA256SUMS").write_text(
            "".join(f"{sha256(data)}  {name}\n" for name, data in files.items()))
        return folder

    def publish(self, *arguments):
        return self.run_script(HERE / "publish.sh", *arguments, keys=PUBLISH_KEYS)

    def writes(self):
        return [call["argv"][4:] for call in self.calls()
                if call["argv"][4:6] in (["s3", "cp"], ["s3api", "put-object"])]

    def test_uploads_the_archive_then_small_files_then_the_manifest(self):
        result = self.publish(self.artifacts())
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.splitlines()[-1],
                         f"MACOS_IMAGE_MANIFEST_PATH=releases/{IMAGE_ID}/manifest.json")
        prefix = f"s3://ciproxy-macos-images/releases/{IMAGE_ID}"
        writes = self.writes()
        self.assertEqual([write[3] for write in writes[:-1]],
                         [f"{prefix}/{name}" for name in ["macos.tar", *SMALL_FILES]])
        manifest = writes[-1]
        self.assertEqual(manifest[:2], ["s3api", "put-object"])
        self.assertEqual(manifest[manifest.index("--key") + 1], f"releases/{IMAGE_ID}/manifest.json")
        self.assertIn("--if-none-match", manifest)
        self.assertEqual(manifest[manifest.index("--if-none-match") + 1], "*")
        size_check = [call["argv"][4:] for call in self.calls()].index(
            ["s3api", "head-object", "--bucket", "ciproxy-macos-images", "--key",
             f"releases/{IMAGE_ID}/macos.tar", "--query", "ContentLength", "--output", "text"])
        self.assertEqual(size_check, len(self.calls()) - 2)
        for call in self.calls():
            self.assert_r2_call(call, KEY_ID, SECRET)
        self.assert_no_key(result)

    def test_refuses_a_release_that_has_a_manifest(self):
        folder = self.artifacts()
        existing = self.bucket / "ciproxy-macos-images" / "releases" / IMAGE_ID
        existing.mkdir(parents=True)
        (existing / "manifest.json").write_text("{}")
        result = self.publish(folder)
        self.assertEqual(result.returncode, 1)
        self.assertIn("manifest.json exists", result.stderr)
        self.assertEqual(self.writes(), [])
        self.assert_no_key(result)

    def test_refuses_a_test_image_without_the_flag(self):
        result = self.publish(self.artifacts(test_image=True))
        self.assertEqual(result.returncode, 1)
        self.assertIn("Give --test-image", result.stderr)
        self.assertEqual(self.calls(), [])
        self.assert_no_key(result)

    def test_publishes_a_test_image_with_the_flag(self):
        result = self.publish("--test-image", self.artifacts(test_image=True))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.writes()[-1][:2], ["s3api", "put-object"])

    def test_stops_before_upload_when_a_checksum_fails(self):
        folder = self.artifacts()
        (folder / "macos.tar").write_bytes(b"changed")
        result = self.publish(folder)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.calls(), [])

    def test_writes_no_manifest_when_the_uploaded_size_differs(self):
        result = self.publish(self.artifacts(size_delta=1))
        self.assertEqual(result.returncode, 1)
        self.assertIn("The release has no manifest", result.stderr)
        self.assertNotIn(["s3api", "put-object"], [write[:2] for write in self.writes()])


XCODE_KEYS = ("R2_XCODE_ACCESS_KEY_ID", "R2_XCODE_SECRET_ACCESS_KEY")


class FetchXcode(FakeR2):
    """Runs a copy of fetch.sh with fake pins, so that every other input is local."""

    def setUp(self):
        super().setUp()
        self.definition = self.root / "definition"
        self.definition.mkdir()
        for name in ("fetch.sh", "r2.sh"):
            shutil.copy(HERE / name, self.definition / name)
        self.work = self.root / "work"
        inputs = self.work / "inputs"
        inputs.mkdir(parents=True)
        packer = io.BytesIO()
        with zipfile.ZipFile(packer, "w") as archive:
            archive.writestr("packer", "fake packer")
        runner_images = io.BytesIO()
        with tarfile.open(fileobj=runner_images, mode="w:gz") as archive:
            archive.addfile(tarfile.TarInfo("runner-images/README.md"), io.BytesIO())
        files = {"restore.ipsw": (b"restore", "RESTORE_IMAGE"),
                 "runner-images.tar.gz": (runner_images.getvalue(), "RUNNER_IMAGES"),
                 "upstream-software.json": (b"{}", "SOFTWARE_MANIFEST"),
                 "runner.tar.gz": (b"runner", "RUNNER"), "packer.zip": (packer.getvalue(), "PACKER")}
        self.xcode = b"xcode archive"
        pins = [f"XCODE_XIP_NAME={PINS['XCODE_XIP_NAME']}", f"XCODE_SHA256={sha256(self.xcode)}",
                "XCODE_VERSION=26.6"]
        for name, (data, pin) in files.items():
            (inputs / name).write_bytes(data)
            pins += [f"{pin}_URL=https://example.invalid/{name}", f"{pin}_SHA256={sha256(data)}"]
        (self.definition / "pins.env").write_text("\n".join(pins) + "\n")
        guest = self.root / "guest"
        guest.mkdir()
        (guest / "install.sh").write_text("#!/bin/sh\n")
        (guest / "install.sh").chmod(0o755)
        self.env["GUEST_DIR"] = str(guest)
        self.object = self.bucket / "ciproxy-xcode" / "xcode" / PINS["XCODE_XIP_NAME"]
        self.object.parent.mkdir(parents=True)
        self.object.write_bytes(self.xcode)

    def fetch(self, keys):
        return self.run_script(self.definition / "fetch.sh", self.work, keys=keys)

    def test_downloads_the_private_xcode_copy_with_the_read_keys(self):
        result = self.fetch(XCODE_KEYS)
        self.assertEqual(result.returncode, 0, result.stderr)
        archive = self.work / "inputs" / "xcode" / PINS["XCODE_XIP_NAME"]
        self.assertEqual([call["argv"][4:] for call in self.calls()], [[
            "s3", "cp", f"s3://ciproxy-xcode/xcode/{PINS['XCODE_XIP_NAME']}", f"{archive}.part"]])
        self.assert_r2_call(self.calls()[0], KEY_ID, SECRET)
        self.assertEqual(archive.read_bytes(), self.xcode)
        self.assertIn(f"{archive}: OK", result.stdout)
        self.assert_no_key(result)

    def test_keeps_the_message_without_keys_or_folder(self):
        result = self.fetch(())
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls(), [])
        self.assertIn("No Xcode archive: only the test image can be built.", result.stdout)


if __name__ == "__main__":
    unittest.main()
