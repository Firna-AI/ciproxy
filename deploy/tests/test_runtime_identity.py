"""Operator UID capture and protected-state promotion at the public CLI."""

import json

from deploy.tests.fake_promotion_tools import install
from deploy.tests.promotion_support import PromotionCase


class RuntimeIdentityTests(PromotionCase):
    def setUp(self):
        super().setUp()
        self.log = install(self)
        self.write_plan("create")

    def test_operator_uid_capture_survives_bootstrap_and_automatic_saved_plan(self):
        path = self.root / ".context/operator/runtime-uids.json"
        expected = {}
        for environment, uid in (("main-preview", "11111111-1111-4111-8111-111111111111"),
                                 ("production", "22222222-2222-4222-8222-222222222222")):
            result = self.cli("namespace_access.py", environment, "--uid-file", str(path))
            self.assertEqual(result.returncode, 0, result.stderr)
            expected[environment] = uid
            self.assertEqual(json.loads(path.read_text()), {"runtime_service_account_uids": expected})
        calls = [json.loads(line) for line in self.log.read_text().splitlines()]
        self.assertEqual([call["args"] for call in calls if call["tool"] == "gcloud"], [
            ["container", "clusters", "get-credentials", "ciproxy-shared", "--region=europe-west1", "--project=ciproxy", "--dns-endpoint"],
            ["container", "clusters", "get-credentials", "ciproxy-shared", "--region=europe-west1", "--project=ciproxy", "--dns-endpoint"],
        ])
        manifests = [call["manifest"] for call in calls if "manifest" in call]
        self.assertEqual([doc["items"][-1]["metadata"]["name"] for doc in manifests], ["control-plane", "control-plane"])
        self.env["FAKE_KSA_UID"] = "33333333-3333-4333-8333-333333333333"
        result = self.cli("namespace_access.py", "production", "--uid-file", str(path))
        self.assertEqual(result.returncode, 0, result.stderr)
        expected["production"] = "33333333-3333-4333-8333-333333333333"
        self.assertEqual(json.loads(path.read_text()), {"runtime_service_account_uids": expected})
        for arguments in (("main-preview", "--runtime-uids", str(path)), ("production",), ("main-preview",)):
            result = self.cli("bootstrap.sh", *arguments)
            self.assertEqual(result.returncode, 0, result.stderr)
            saved = self.root / ".context/deploy/ciproxy/runtime-uids.tfvars.json"
            self.assertEqual(json.loads(saved.read_text()), {"runtime_service_account_uids": expected})
        self.trusted("production")
        result = self.cli("terraform.py", "plan", "production", "bootstrap")
        self.assertEqual(result.returncode, 0, result.stderr)
        inputs = self.root / ".context/plans/production/bootstrap/inputs.tfvars.json"
        self.assertEqual(json.loads(inputs.read_text()), {
            "project_id": "ciproxy", "production_enabled": True, "runtime_service_account_uids": expected,
        })
        result = self.cli("terraform.py", "apply", "production", "bootstrap")
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()]
        final = [call for call in calls if call["tool"] == "terraform" and call["args"][1] == "apply"][-1]
        self.assertTrue(final["args"][-1].endswith("/release.tfplan"))

    def test_missing_operator_uid_never_replaces_trusted_input(self):
        path = self.root / ".context/operator-uids.json"
        original = '{"runtime_service_account_uids":{"production":"trusted-uid"}}\n'
        path.write_text(original)
        self.env["FAKE_KSA_UID"] = ""
        result = self.cli("namespace_access.py", "production", "--uid-file", str(path))
        self.assertEqual(result.returncode, 1)
        self.assertIn("No trust input was written", result.stderr)
        self.assertEqual(path.read_text(), original)
