#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check write-manifest.py offline: it refuses a variant and test-image pair
that disagrees, before it reads or writes any file, and it records the disk
that the image has."""

import importlib.util
import json
import pathlib
import subprocess
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
SCRIPT = HERE / "write-manifest.py"
spec = importlib.util.spec_from_file_location("write_manifest", SCRIPT)
manifest = importlib.util.module_from_spec(spec)
spec.loader.exec_module(manifest)


def run(*arguments):
    folder = tempfile.mkdtemp()
    result = subprocess.run([sys.executable, str(SCRIPT), folder, "--guest-binary",
                             "/nonexistent", *arguments], capture_output=True, text=True)
    return result, pathlib.Path(folder)


class VariantPair(unittest.TestCase):
    def test_pairs_that_agree_pass(self):
        for variant, test_image in (("full", False), ("standard", False), ("test", True)):
            with self.subTest(variant):
                manifest.check_variant(variant, test_image)

    def test_pairs_that_disagree_fail(self):
        for variant, test_image in (("full", True), ("standard", True), ("test", False)):
            with self.subTest(variant), self.assertRaises(SystemExit):
                manifest.check_variant(variant, test_image)

    def test_the_script_refuses_before_any_file(self):
        for arguments in (["--variant", "full", "--test-image"], ["--variant", "test"]):
            with self.subTest(arguments):
                result, folder = run(*arguments)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("disagree", result.stderr)
                self.assertEqual(list(folder.iterdir()), [])


class DiskSize(unittest.TestCase):
    def test_the_inputs_record_the_disk_that_the_image_has(self):
        pins = dict(line.split("=", 1) for line in (HERE / "pins.env").read_text().splitlines()
                    if line and not line.startswith("#"))
        version, build = pins["MACOS_TARGET"].split("-")
        folder = pathlib.Path(tempfile.mkdtemp())
        (folder / "bundle").mkdir()
        with open(folder / "bundle/disk.img", "wb") as disk:
            disk.truncate(123456789)
        (folder / "bundle/aux.img").write_bytes(b"aux")
        (folder / "bundle/hardware-model.bin").write_bytes(b"model")
        (folder / "guest").write_bytes(b"guest binary")
        (folder / "guest-facts.json").write_text(json.dumps({
            "macos_version": version, "macos_build": build, "data_volume_used_bytes": 1,
            "free_job_bytes": 2, "sip": "disabled", "xcode": "none"}))
        result = subprocess.run([sys.executable, str(SCRIPT), str(folder), "--guest-binary",
                                 str(folder / "guest"), "--variant", "standard"],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        written = json.loads((folder / "manifest.json").read_text())
        self.assertEqual(written["root_disk_bytes"], 123456789)
        self.assertEqual(written["inputs"]["DISK_BYTES"], "123456789")
        self.assertEqual(written["inputs"]["STANDARD_DISK_BYTES"], pins["STANDARD_DISK_BYTES"])
        self.assertEqual(written["variant"], "standard")


if __name__ == "__main__":
    unittest.main()
