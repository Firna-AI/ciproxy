#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check publish.sh offline. check-r2-xcode.py checks the Xcode download.

A fake `aws` on PATH records each call and keeps the bucket in a folder.
Needs bash and shasum.
"""

import importlib.util
import json
import pathlib
import shutil
import unittest

HERE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("r2_fake", HERE / "r2_fake.py")
r2_fake = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r2_fake)
FakeR2, sha256, KEY_ID, SECRET = r2_fake.FakeR2, r2_fake.sha256, r2_fake.KEY_ID, r2_fake.SECRET
IMAGE_ID = "macos26-arm64-check-v1"
PINS = dict(line.split("=", 1) for line in (HERE / "pins.env").read_text().splitlines()
            if line and not line.startswith("#"))
PUBLISH_KEYS = ("R2_PUBLISH_ACCESS_KEY_ID", "R2_PUBLISH_SECRET_ACCESS_KEY")
SMALL_FILES = ["downloads.json", "SHA256SUMS", "measurement.json", "software-report.json",
               "upstream-software.json", "size-report.txt"]


class Publish(FakeR2):
    def artifacts(self, test_image=False, size_delta=0, image_id=IMAGE_ID, variant="full",
                  size_bytes=None):
        """variant None leaves the key out, as an older build does."""
        folder = self.root / "artifacts"
        folder.mkdir()
        tar = b"macos archive" * 1000
        files = {"macos.tar": tar, "measurement.json": b"{}", "software-report.json": b"{}",
                 "upstream-software.json": b"{}", "size-report.txt": b"macos.tar: 13000\n",
                 "manifest.json": json.dumps({"image_id": image_id, "test_image": test_image,
                                              **({} if variant is None else
                                                 {"variant": variant})}).encode(),
                 "downloads.json": json.dumps({"macos": {
                     "file": "macos.tar", "sha256": sha256(tar),
                     "size_bytes": size_bytes or len(tar) + size_delta}}).encode()}
        for name, data in files.items():
            (folder / name).write_bytes(data)
        (folder / "SHA256SUMS").write_text(
            "".join(f"{sha256(data)}  {name}\n" for name, data in files.items()))
        return folder

    def publish(self, *arguments, env=None, trace=False):
        return self.run_script(HERE / "publish.sh", *arguments, keys=PUBLISH_KEYS, env=env,
                               trace=trace)

    def writes(self):
        return [call["argv"][4:] for call in self.calls()
                if call["argv"][4:6] in (["s3", "cp"], ["s3api", "put-object"])]

    def test_uploads_the_archive_then_small_files_then_the_manifest(self):
        # A bash -x trace shows no key either.
        result = self.publish(self.artifacts(), trace=True)
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

    def test_stops_when_the_release_check_fails_with_another_error(self):
        forbidden = "An error occurred (403) when calling the HeadObject operation: Forbidden"
        result = self.publish(self.artifacts(), env={"FAKE_AWS_HEAD_ERROR": forbidden})
        self.assertEqual(result.returncode, 1)
        self.assertIn(f"Could not check s3://ciproxy-macos-images/releases/{IMAGE_ID}/manifest.json",
                      result.stderr)
        self.assertEqual(self.writes(), [])

    def test_keeps_the_manifest_of_a_publish_that_wrote_it_first(self):
        key = f"releases/{IMAGE_ID}/manifest.json"
        result = self.publish(self.artifacts(),
                              env={"FAKE_AWS_OTHER_MANIFEST": f"ciproxy-macos-images/{key}"})
        self.assertEqual(result.returncode, 1)
        self.assertIn("When another publish wrote it first, the release exists.", result.stderr)
        self.assertEqual((self.bucket / "ciproxy-macos-images" / key).read_text(),
                         "other publisher")

    def test_refuses_an_aws_cli_older_than_the_conditional_write_release(self):
        empty = self.root / "empty"
        empty.mkdir()
        for version, refused in [("1.33.0", True), ("2.17.33", True), ("2.17.34", False),
                                 ("2.100.0", False)]:
            with self.subTest(version=version):
                result = self.publish(empty, env={"FAKE_AWS_VERSION": version})
                self.assertEqual("Install version 2.17.34 or later" in result.stderr, refused,
                                 result.stderr)
                self.assertEqual("has no macos.tar" in result.stderr, not refused, result.stderr)

    def test_refuses_an_image_id_that_cannot_name_a_folder(self):
        for image_id in ["", "../x"]:
            with self.subTest(image_id=image_id):
                folder = self.artifacts(image_id=image_id)
                result = self.publish(folder)
                shutil.rmtree(folder)
                self.assertEqual(result.returncode, 1)
                self.assertIn("manifest.json has no valid image_id", result.stderr)
                self.assertEqual(self.calls(), [])

    def test_names_a_missing_publish_key(self):
        result = self.run_script(HERE / "publish.sh", self.artifacts(),
                                 keys=("R2_PUBLISH_ACCESS_KEY_ID",))
        self.assertEqual(result.returncode, 1)
        self.assertIn("Set R2_PUBLISH_SECRET_ACCESS_KEY.", result.stderr)
        self.assertEqual(self.calls(), [])
        self.assert_no_key(result)

    def test_publishes_the_full_and_the_standard_release_images(self):
        for image_id, variant, size in [(PINS["IMAGE_ID"], "full", 73914951680),
                                        (PINS["STANDARD_IMAGE_ID"], "standard", 47559669760)]:
            with self.subTest(variant=variant):
                folder = self.artifacts(image_id=image_id, variant=variant, size_bytes=size)
                result = self.publish(folder, env={"FAKE_AWS_SIZE": str(size)})
                shutil.rmtree(folder)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn(f"Publishing {image_id}, variant {variant}.", result.stdout)
                self.assertEqual(result.stdout.splitlines()[-2:], [
                    f"Published {image_id}, variant {variant}.",
                    f"MACOS_IMAGE_MANIFEST_PATH=releases/{image_id}/manifest.json"])

    def test_refuses_an_unknown_variant_or_one_that_disagrees_with_test_image(self):
        disagrees = "manifest.json says variant {} and test_image"
        for variant, test_image, message in [
                ("standard", True, disagrees.format("standard")),
                ("full", True, disagrees.format("full")),
                ("test", False, disagrees.format("test")),
                ("huge", False, "manifest.json has an unknown variant: 'huge'")]:
            with self.subTest(variant=variant, test_image=test_image):
                folder = self.artifacts(variant=variant, test_image=test_image)
                result = self.publish("--test-image", folder)
                shutil.rmtree(folder)
                self.assertEqual(result.returncode, 1)
                self.assertIn(message, result.stderr)
                self.assertEqual(self.calls(), [])

    def test_counts_an_older_build_without_variant_by_its_test_flag(self):
        for test_image, variant in [(False, "full"), (True, "test")]:
            with self.subTest(variant=variant):
                image_id = f"{IMAGE_ID}-{variant}"
                folder = self.artifacts(variant=None, test_image=test_image, image_id=image_id)
                arguments = ["--test-image"] if test_image else []
                result = self.publish(*arguments, folder)
                shutil.rmtree(folder)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stderr.count("manifest.json has no variant"), 1)
                self.assertIn(f"It counts as {variant}.", result.stderr)
                self.assertIn(f"Published {image_id}, variant {variant}.", result.stdout)

    def test_refuses_a_test_image_without_the_flag(self):
        result = self.publish(self.artifacts(test_image=True, variant="test"))
        self.assertEqual(result.returncode, 1)
        self.assertIn("Give --test-image", result.stderr)
        self.assertEqual(self.calls(), [])
        self.assert_no_key(result)

    def test_publishes_a_test_image_with_the_flag(self):
        result = self.publish("--test-image", self.artifacts(test_image=True, variant="test"))
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


if __name__ == "__main__":
    unittest.main()
