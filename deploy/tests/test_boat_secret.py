"""Deliver the optional Boat snapshot through the real deploy command."""

import base64
import json
import shutil
import subprocess

from deploy.tests.fake_promotion_tools import install
from deploy.tests.promotion_support import PromotionCase, ROOT


class BoatSecretTests(PromotionCase):
    def setUp(self):
        super().setUp()
        self.log = install(self)
        self.snapshot = "fixture-boat-share-code"
        self.env.update({
            "ALERT_EMAIL": "deploy@example.invalid",
            "RELEASE_SERVER_IMAGE_DIGEST": "sha256:" + "a" * 64,
            "CONTROL_PLANE_RUNTIME_READY": "true",
            "CONTROL_PLANE_IMAGE": "example.invalid/control@sha256:" + "b" * 64,
            "FAKE_BOAT_SNAPSHOT": self.snapshot,
        })
        chart = self.root / "deploy/helm/ciproxy"
        shutil.copytree(ROOT / "deploy/helm/ciproxy", chart)
        # Helm parses its own rendered deployment into JSON for assertions.
        (chart / "templates/probe.yaml").write_text(
            '{{ include (print $.Template.BasePath "/control-plane.yaml") . | fromYaml | toJson }}\n'
        )

    def calls(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()]

    def snapshots(self):
        return [call["manifest"] for call in self.calls() if "manifest" in call
                and call["manifest"]["metadata"]["name"] == "boat-shared-snapshot"]

    def values_path(self):
        helm = next(call for call in reversed(self.calls())
                    if call["tool"] == "helm" and "deploy/helm/ciproxy" in call["args"])
        return self.root / next(argument.split("=", 1)[1] for argument in helm["args"]
                                if argument.startswith("--values="))

    def pod_template(self):
        rendered = subprocess.run([
            "helm", "template", "ciproxy", str(self.root / "deploy/helm/ciproxy"),
            "--values", str(self.values_path()), "--show-only", "templates/probe.yaml",
        ], text=True, capture_output=True, cwd=ROOT)
        self.assertEqual(rendered.returncode, 0, rendered.stderr)
        document = json.loads(rendered.stdout[rendered.stdout.index("{"):])
        return document["spec"]["template"]

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
