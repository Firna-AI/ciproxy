"""Deliver the optional R2 read key of the macOS images through the real deploy command."""

import base64
import json

from deploy.tests.optional_secret_support import OptionalSecretCase


KEY = '{"access_key_id": "fixture-r2-id", "secret_access_key": "fixture-r2-secret"}'


class R2ImagesSecretTests(OptionalSecretCase):
    def setUp(self):
        super().setUp()
        self.env["FAKE_R2_KEY"] = KEY

    def test_version_reaches_the_optional_pod_variable_and_rotation_restarts_the_pod(self):
        first = self.cli("deploy.py", "main-preview")
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual([secret["data"] for secret in self.secrets("r2-images")], [{
            "R2_IMAGES_KEY": base64.b64encode(KEY.encode()).decode(),
        }])
        self.assertEqual(self.variable("R2_IMAGES_KEY"), {
            "name": "R2_IMAGES_KEY", "valueFrom": {"secretKeyRef": {
                "name": "r2-images", "key": "R2_IMAGES_KEY", "optional": True,
            }},
        })
        for private_value in ("fixture-r2-secret", base64.b64encode(KEY.encode()).decode()):
            self.assertNotIn(private_value, self.values_path().read_text())
            self.assertNotIn(private_value, first.stdout + first.stderr)
            self.assertNotIn(private_value, json.dumps([call["args"] for call in self.calls()]))
        revision = self.pod_template()["metadata"]["annotations"]["checksum/r2-images"]
        self.env.update(FAKE_R2_REVISION="52")
        second = self.cli("deploy.py", "main-preview")
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertNotEqual(self.pod_template()["metadata"]["annotations"]["checksum/r2-images"], revision)

    def test_no_version_applies_an_empty_secret_and_the_deploy_continues(self):
        self.env["FAKE_R2_ERROR"] = "NOT_FOUND"
        result = self.cli("deploy.py", "main-preview")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([secret["data"] for secret in self.secrets("r2-images")], [{}])
        self.assertEqual(self.variable("R2_IMAGES_KEY")["valueFrom"]["secretKeyRef"]["optional"], True)

    def test_access_denial_stops_the_deploy_before_helm(self):
        self.env["FAKE_R2_ERROR"] = "PERMISSION_DENIED"
        result = self.cli("deploy.py", "main-preview")
        self.assertEqual(result.returncode, 1)
        self.assertIn("R2 images key secret read failed", result.stderr)
        self.assertEqual([call for call in self.calls() if call["tool"] == "helm"], [])

    def test_secret_read_runs_with_cloud_sdk_body_logging_off(self):
        self.env.update(CLOUDSDK_CORE_LOG_HTTP="true", CLOUDSDK_CORE_DISABLE_FILE_LOGGING="false")
        result = self.cli("deploy.py", "main-preview")
        self.assertEqual(result.returncode, 0, result.stderr)
        reads = [call for call in self.calls() if call["tool"] == "gcloud"
                 and "--secret=fake-r2-images" in call["args"]]
        self.assertEqual([read["sdk_logging"] for read in reads], [{
            "CLOUDSDK_CORE_LOG_HTTP": "false", "CLOUDSDK_CORE_DISABLE_FILE_LOGGING": "true",
        }])
