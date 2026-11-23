#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check offline that write-manifest.py refuses a variant and test-image pair
that disagrees, before it reads or writes any file."""

import importlib.util
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


if __name__ == "__main__":
    unittest.main()
