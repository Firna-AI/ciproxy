"""Deliver the optional Boat snapshot through the real deploy command."""

import base64
import json

from deploy.tests.optional_secret_support import OptionalSecretCase


class BoatSecretTests(OptionalSecretCase):
    def setUp(self):
        super().setUp()
        self.snapshot = "fixture-boat-share-code"
        self.env["FAKE_BOAT_SNAPSHOT"] = self.snapshot

    def snapshots(self):
        return self.secrets("boat-shared-snapshot")

    def test_overflow_kms_setting_uses_the_deployed_key(self):
        result = self.cli("deploy.py", "main-preview")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.variable("OVERFLOW_KMS_KEY"), {
            "name": "OVERFLOW_KMS_KEY", "value": "test-key",
        })
        self.assertIsNone(self.variable("BOAT_KMS_KEY"))

    def test_version_is_synced_and_rotation_restarts_the_pod(self):
        first = self.cli("deploy.py", "main-preview")
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual([secret["data"] for secret in self.snapshots()], [{
            "BOAT_SHARED_SNAPSHOT": base64.b64encode(self.snapshot.encode()).decode(),
        }])
        template = self.pod_template()
        variable = next((item for item in template["spec"]["containers"][0]["env"]
                         if item["name"] == "BOAT_SHARED_SNAPSHOT"), None)
        self.assertEqual(variable, {
            "name": "BOAT_SHARED_SNAPSHOT", "valueFrom": {"secretKeyRef": {
                "name": "boat-shared-snapshot", "key": "BOAT_SHARED_SNAPSHOT", "optional": True,
            }},
        })
        revision = template["metadata"]["annotations"]["checksum/boat-shared-snapshot"]
        self.env.update(FAKE_BOAT_SNAPSHOT="fixture-new-share-code", FAKE_BOAT_REVISION="42")
        second = self.cli("deploy.py", "main-preview")
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertEqual(self.snapshots()[-1]["data"], {
            "BOAT_SHARED_SNAPSHOT": base64.b64encode(b"fixture-new-share-code").decode(),
        })
        self.assertNotEqual(self.pod_template()["metadata"]["annotations"]["checksum/boat-shared-snapshot"], revision)

    def test_no_version_clears_the_key_and_keeps_the_pod_value_unset(self):
        existing = self.cli("deploy.py", "main-preview")
        self.assertEqual(existing.returncode, 0, existing.stderr)
        self.env["FAKE_BOAT_ERROR"] = "NOT_FOUND"
        missing = self.cli("deploy.py", "main-preview")
        self.assertEqual(missing.returncode, 0, missing.stderr)
        self.assertEqual([secret["data"] for secret in self.snapshots()][-1:], [{}])
        variables = self.pod_template()["spec"]["containers"][0]["env"]
        variable = next((item for item in variables if item["name"] == "BOAT_SHARED_SNAPSHOT"), {})
        self.assertEqual(variable.get("valueFrom", {}).get("secretKeyRef"), {
            "name": "boat-shared-snapshot", "key": "BOAT_SHARED_SNAPSHOT", "optional": True,
        })
        self.assertNotIn("value", variable)

    def test_snapshot_never_reaches_values_arguments_files_or_logs(self):
        result = self.cli("deploy.py", "main-preview")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.snapshots()), 1, "The private delivery must actually run.")
        encoded = base64.b64encode(self.snapshot.encode()).decode()
        values = self.values_path().read_text()
        manifests = [call for call in self.calls() if "manifest" in call
                     and call["manifest"]["metadata"]["name"] == "boat-shared-snapshot"]
        self.assertEqual(manifests[0]["args"], [
            "apply", "--server-side", "--field-manager=ciproxy-deploy", "-f", "-",
            "-o", "jsonpath={.metadata.resourceVersion}",
        ])
        self.assertEqual(manifests[0]["manifest"]["metadata"], {
            "name": "boat-shared-snapshot", "namespace": "ciproxy-main-preview",
        })
        for private_value in (self.snapshot, encoded):
            self.assertNotIn(private_value, values)
            self.assertNotIn(private_value, result.stdout + result.stderr)
            self.assertNotIn(private_value, json.dumps([call["args"] for call in self.calls()]))
            for path in (self.root / ".context/deploy").rglob("*"):
                if path.is_file():
                    self.assertNotIn(private_value, path.read_text(), str(path))

    def test_access_denial_stops_deploy_instead_of_disabling_boat(self):
        self.env["FAKE_BOAT_ERROR"] = "PERMISSION_DENIED"
        result = self.cli("deploy.py", "main-preview")
        self.assertEqual(result.returncode, 1)
        self.assertIn("Boat snapshot secret read failed", result.stderr)
        self.assertEqual(self.snapshots(), [])
        self.assertEqual([call for call in self.calls() if call["tool"] == "helm"], [])

    def test_apply_failure_does_not_log_the_secret_payload(self):
        self.env["FAKE_BOAT_APPLY_ERROR"] = "true"
        result = self.cli("deploy.py", "main-preview")
        self.assertEqual(result.returncode, 1)
        self.assertIn("Boat snapshot secret apply failed", result.stderr)
        self.assertNotIn(self.snapshot, result.stdout + result.stderr)
        self.assertNotIn(base64.b64encode(self.snapshot.encode()).decode(), result.stdout + result.stderr)
