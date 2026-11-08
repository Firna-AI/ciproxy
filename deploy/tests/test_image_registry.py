"""Registry reuse, immutable publication, and private credential cleanup."""

import json
from pathlib import Path

from deploy.tests.image_support import ImageCase, IMAGE, REPOSITORY


class ImageRegistryTests(ImageCase):
    def test_lookup_reuses_digest_and_explicit_rollback(self):
        source_hash = "b" * 64
        self.state.write_text(json.dumps({REPOSITORY + ":source-" + source_hash: "sha256:" + "a" * 64}))
        for options in ([], ["--rollback", IMAGE]):
            result = self.image("lookup", "--source-hash", source_hash, *options)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout, "image=" + IMAGE + "\n")
        calls = self.calls()
        logins = [call for call in calls if call["args"][:2] == ["auth", "login"]]
        self.assertEqual(len(logins), 2)
        for call in logins:
            self.assertTrue(call["token_on_stdin"])
            self.assertEqual(call["config_mode"], 0o700)
            self.assertFalse(Path(call["config"]).exists())

    def test_only_manifest_not_found_requests_a_build(self):
        for error, expected in (("MANIFEST_UNKNOWN: manifest unknown", 0),
                                ("DENIED: fixture-token-not-a-secret", 1),
                                ("NAME_UNKNOWN: image missing", 0),
                                ("503 service unavailable: MANIFEST_UNKNOWN", 1),
                                ("DENIED: MANIFEST_UNKNOWN", 1),
                                ("503 service unavailable", 1)):
            with self.subTest(registry_error=error.split(":")[0]):
                self.env["IMAGE_LOOKUP_ERROR"] = error
                result = self.image("lookup", "--source-hash", "b" * 64)
                self.assertEqual(result.returncode, expected, result.stderr)
                self.assertEqual(result.stdout, "image=\n" if expected == 0 else "")
                self.assertNotIn("fixture-token-not-a-secret", result.stdout + result.stderr)
        self.env.pop("IMAGE_LOOKUP_ERROR")
        for error in ("NOT_FOUND: repository missing", "PERMISSION_DENIED: fixture-token-not-a-secret", "503 unavailable"):
            self.env["IMAGE_REPOSITORY_ERROR"] = error
            result = self.image("lookup", "--source-hash", "b" * 64)
            self.assertEqual(result.returncode, 1)
            self.assertEqual(result.stdout, "")
            self.assertNotIn("fixture-token-not-a-secret", result.stderr)

    def test_publish_pushes_digest_then_tags_and_cleans_credentials(self):
        archive = self.root / ".context/image.tar"
        archive.write_bytes(b"fixture-image")
        result = self.image("publish", "--source-hash", "b" * 64, "--archive", str(archive),
                            "--expected-digest", "sha256:" + "a" * 64)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "image=" + IMAGE + "\n")
        calls = self.calls()
        push = next(call for call in calls if call["args"][0] == "push")
        self.assertEqual(push["args"], ["push", str(archive), IMAGE])
        tag = next(call for call in calls if call["args"][0] == "tag")
        self.assertEqual(tag["args"], ["tag", IMAGE, "source-" + "b" * 64])
        login = next(call for call in calls if call["args"][:2] == ["auth", "login"])
        self.assertTrue(login["token_on_stdin"])
        self.assertEqual(login["config_mode"], 0o700)
        self.assertFalse(Path(login["config"]).exists())
        self.assertNotIn("fixture-token-not-a-secret", result.stdout + result.stderr + self.log.read_text())

    def test_reused_publish_needs_no_cloud_authentication(self):
        result = self.image("publish", "--source-hash", "b" * 64, "--existing", IMAGE)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "image=" + IMAGE + "\n")
        self.assertEqual(self.calls(), [])

    def test_authentication_and_publish_failures_hide_payload_and_remove_config(self):
        archive = self.root / ".context/image.tar"
        archive.write_bytes(b"fixture-image")
        for command in ("gcloud auth print-access-token", "crane auth login", "crane push " + str(archive)):
            with self.subTest(command=command.split()[0:2]):
                self.env["IMAGE_FAIL_COMMAND"] = command
                result = self.image("publish", "--source-hash", "b" * 64, "--archive", str(archive),
                                    "--expected-digest", "sha256:" + "a" * 64)
                self.assertEqual(result.returncode, 1)
                self.assertEqual(result.stdout, "")
                self.assertNotIn("fixture-token-not-a-secret", result.stderr)
                for call in self.calls():
                    if call["config"]:
                        self.assertFalse(Path(call["config"]).exists())

    def test_archive_substitution_fails_before_registry_access(self):
        archive = self.root / ".context/substituted-image.tar"
        archive.write_bytes(b"substituted-image")
        result = self.image("publish", "--source-hash", "b" * 64, "--archive", str(archive),
                            "--expected-digest", "sha256:" + "b" * 64)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stdout, "")
        self.assertIn("differs from the build job's image digest", result.stderr)
        self.assertEqual([call["tool"] for call in self.calls()], ["crane"])
        self.assertEqual(self.calls()[0]["args"], ["digest", "--tarball", str(archive)])
