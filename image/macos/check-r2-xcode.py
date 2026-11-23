#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check the R2 Xcode download of fetch.sh offline.

A copy of fetch.sh runs with fake pins, so that every input but Xcode is
local, and a fake `aws` on PATH keeps the bucket in a folder. Needs bash and
shasum.
"""

import importlib.util
import io
import pathlib
import shutil
import tarfile
import unittest
import zipfile

HERE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("r2_fake", HERE / "r2_fake.py")
r2_fake = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r2_fake)
FakeR2, sha256, KEY_ID, SECRET = r2_fake.FakeR2, r2_fake.sha256, r2_fake.KEY_ID, r2_fake.SECRET
PINS = dict(line.split("=", 1) for line in (HERE / "pins.env").read_text().splitlines()
            if line and not line.startswith("#"))
XCODE_KEYS = ("R2_XCODE_ACCESS_KEY_ID", "R2_XCODE_SECRET_ACCESS_KEY")


class FetchXcode(FakeR2):
    """Runs copies of fetch.sh and build.sh with fake pins, so that every other input
    is local."""

    def setUp(self):
        super().setUp()
        self.definition = self.root / "definition"
        self.definition.mkdir()
        for name in ("fetch.sh", "r2.sh", "build.sh", "build-lib.sh"):
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
                "XCODE_VERSION=26.6", "IMAGE_ID=check-full-v1", "DISK_BYTES=1000000"]
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
        self.archive = self.work / "inputs" / "xcode" / PINS["XCODE_XIP_NAME"]

    def fetch(self, keys, env=None, trace=False):
        return self.run_script(self.definition / "fetch.sh", self.work, keys=keys, env=env,
                               trace=trace)

    def test_downloads_the_private_xcode_copy_with_the_read_keys(self):
        # A bash -x trace shows no key either.
        result = self.fetch(XCODE_KEYS, trace=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        archive = self.archive
        self.assertEqual([call["argv"][4:] for call in self.calls()], [[
            "s3", "cp", f"s3://ciproxy-xcode/xcode/{PINS['XCODE_XIP_NAME']}", f"{archive}.part"]])
        self.assert_r2_call(self.calls()[0], KEY_ID, SECRET)
        self.assertEqual(archive.read_bytes(), self.xcode)
        self.assertIn(f"{archive}.part: OK", result.stdout)
        self.assert_no_key(result)
        result = self.fetch(XCODE_KEYS)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.calls()), 1)

    def test_downloads_with_any_aws_cli_and_warns_on_one_that_is_not_v2(self):
        for version, warns in [("2.15.0", False), ("1.33.0", True)]:
            with self.subTest(version=version):
                self.archive.unlink(missing_ok=True)
                result = self.fetch(XCODE_KEYS, env={"FAKE_AWS_VERSION": version})
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(self.archive.read_bytes(), self.xcode)
                self.assertEqual(f"Warning: the AWS CLI is {version}." in result.stderr, warns)

    def test_keeps_the_message_without_keys_or_folder(self):
        result = self.fetch(())
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls(), [])
        self.assertIn("No Xcode archive: only the test image can be built.", result.stdout)
        self.assertIn("Or source the R2 Xcode env file", result.stdout)

    def test_names_the_missing_key_when_only_some_are_set(self):
        result = self.fetch(("R2_XCODE_ACCESS_KEY_ID",))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls(), [])
        self.assertIn("The R2 Xcode download also needs R2_XCODE_SECRET_ACCESS_KEY.", result.stderr)
        self.assert_no_key(result)

    def test_prefers_the_local_xcode_folder_over_the_bucket(self):
        folder = self.root / "xcode"
        folder.mkdir()
        (folder / PINS["XCODE_XIP_NAME"]).write_bytes(self.xcode)
        result = self.fetch(XCODE_KEYS, env={"XCODE_DIR": str(folder)})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls(), [])
        self.assertEqual(self.archive.read_bytes(), self.xcode)

    def test_refuses_a_bad_existing_archive_and_a_bad_local_copy(self):
        folder = self.root / "xcode"
        folder.mkdir()
        (folder / PINS["XCODE_XIP_NAME"]).write_bytes(b"bad local copy")
        for source, env in [("existing", {}), ("XCODE_DIR", {"XCODE_DIR": str(folder)})]:
            with self.subTest(source=source):
                if source == "existing":
                    self.archive.parent.mkdir(parents=True, exist_ok=True)
                    self.archive.write_bytes(b"bad existing archive")
                result = self.fetch(XCODE_KEYS, env=env)
                self.archive.unlink(missing_ok=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("FAILED", result.stdout)
                self.assertEqual(self.calls(), [])

    def test_build_gives_the_xcode_keys_to_the_fetch_phase_only(self):
        tool = self.work / "tools" / "ciproxy-image-vm"
        tool.parent.mkdir()
        tool.write_text("#!/bin/sh\nenv | sed -n 's/^\\(R2_[A-Z_]*\\)=.*/\\1/p' | sort > \"$0.env\"\n")
        tool.chmod(0o755)
        keys = (*XCODE_KEYS, "R2_PUBLISH_ACCESS_KEY_ID", "R2_PUBLISH_SECRET_ACCESS_KEY")
        result = self.run_script(self.definition / "build.sh", self.work, "fetch", "install",
                                 keys=keys, trace=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.calls()), 1)
        self.assert_r2_call(self.calls()[0], KEY_ID, SECRET)
        self.assertEqual(self.archive.read_bytes(), self.xcode)
        self.assertEqual(pathlib.Path(f"{tool}.env").read_text().split(), ["R2_ACCOUNT_ID"])
        self.assert_no_key(result)

    def test_build_reports_a_failed_fetch_phase_once(self):
        result = self.run_script(self.definition / "build.sh", self.work, "fetch",
                                 keys=XCODE_KEYS, env={"GUEST_DIR": str(self.root / "none")})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stderr.count("build.sh: phase fetch failed"), 1, result.stderr)

    def test_downloads_again_after_a_bad_download(self):
        self.object.write_bytes(b"broken download")
        result = self.fetch(XCODE_KEYS)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.archive.exists(), False)
        self.object.write_bytes(self.xcode)
        result = self.fetch(XCODE_KEYS)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.calls()), 2)
        self.assertEqual(self.archive.read_bytes(), self.xcode)


if __name__ == "__main__":
    unittest.main()
