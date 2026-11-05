"""Shared cloud promotion keeps preview credentials outside production authority."""

import json

from deploy.tests.fake_promotion_tools import install
from deploy.tests.promotion_support import PromotionCase


class SharedPromotionTests(PromotionCase):
    def setUp(self):
        super().setUp()
        self.log = install(self)
        self.write_plan("create")

    def test_callable_root_routing_follows_launch_and_protected_main(self):
        for environment, ready, switch, ref, expected in (
            ("main-preview", "false", "false", "refs/heads/main", ["seed bootstrap", "shared", "foundation", "edge"]),
            ("main-preview", "false", "false", "refs/heads/calummoore/ci-proxy-load-balancer", ["", "shared", "foundation", "edge"]),
            ("main-preview", "true", "true", "refs/heads/main", ["", "", "foundation", ""]),
            ("main-preview", "true", "false", "refs/heads/main", ["", "", "foundation", ""]),
            ("production", "true", "true", "refs/heads/main", ["seed bootstrap", "shared", "foundation", "edge"]),
        ):
            with self.subTest(environment=environment, ready=ready, switch=switch, ref=ref):
                self.trusted(environment)
                self.env.update(FAKE_PRODUCTION_READY=ready, PRODUCTION_DEPLOY_ENABLED=switch,
                                GITHUB_REF=ref, GITHUB_WORKFLOW_REF=f"Firna-AI/ciproxy/.github/workflows/deploy-{environment}.yml@{ref}")
                for scope, selected in zip(("trust", "shared", "foundation", "edge"), expected):
                    result = self.cli("ci_roots.py", environment, scope)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    self.assertEqual(result.stdout.strip(), selected)

    def test_empty_scopes_publish_no_apply_inputs_and_need_no_plan_artifact(self):
        for ready, ref, scope in (
            ("false", "refs/heads/calummoore/ci-proxy-load-balancer", "trust"),
            ("true", "refs/heads/main", "trust"),
            ("true", "refs/heads/main", "shared"),
            ("true", "refs/heads/main", "data"),
            ("true", "refs/heads/main", "edge"),
        ):
            with self.subTest(ready=ready, scope=scope):
                self.trusted()
                self.env.update(FAKE_PRODUCTION_READY=ready, PRODUCTION_DEPLOY_ENABLED="true" if ready == "true" else "false", GITHUB_REF=ref,
                                GITHUB_WORKFLOW_REF="Firna-AI/ciproxy/.github/workflows/deploy-main-preview.yml@" + ref)
                self.env.pop("CIPROXY_REUSABLE_APPLY", None)
                output = self.root / ".context/empty-route-output"
                output.unlink(missing_ok=True)
                self.env["GITHUB_OUTPUT"] = str(output)
                self.log.write_text("")
                result = self.cli("ci_roots.py", "main-preview", scope)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout.strip(), "")
                self.assertEqual(dict(line.split("=", 1) for line in output.read_text().splitlines()),
                                 {"roots": "", "target": "main-preview"})
                calls = [json.loads(line) for line in self.log.read_text().splitlines()]
                self.assertEqual([call["tool"] for call in calls], ["gcloud", "gcloud"])
                self.assertFalse((self.root / ".context/plans").exists())
                self.assertFalse((self.root / ".context/deploy").exists())

    def test_unsafe_switch_and_preview_central_write_stop_before_terraform(self):
        self.trusted()
        self.env.update(PRODUCTION_DEPLOY_ENABLED="true", FAKE_PRODUCTION_READY="false")
        result = self.cli("ci_roots.py", "main-preview", "shared")
        self.assertEqual(result.returncode, 1)
        self.assertIn("before the GCP launch handoff", result.stderr)
        self.env.update(FAKE_PRODUCTION_READY="true", PRODUCTION_DEPLOY_ENABLED="false")
        self.env["FAKE_TRUST_DISABLED"] = "false"
        result = self.cli("ci_roots.py", "main-preview", "shared")
        self.assertEqual(result.returncode, 1)
        self.assertIn("handoff is incomplete", result.stderr)
        self.env.pop("FAKE_TRUST_DISABLED")
        result = self.cli("terraform.py", "plan", "main-preview", "shared")
        self.assertEqual(result.returncode, 1)
        self.assertIn("reads central shared", result.stderr)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()]
        self.assertEqual([call for call in calls if call["tool"] == "terraform"], [])

    def test_preview_bootstrap_after_launch_never_enables_retired_accounts(self):
        result = self.cli("bootstrap.sh", "production")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.log.write_text("")
        result = self.cli("bootstrap.sh", "main-preview")
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()]
        applies = [call["args"] for call in calls if call["tool"] == "terraform" and call["args"][1] == "apply"]
        self.assertEqual(len(applies), 1)
        self.assertIn("-var=production_enabled=true", applies[0])
        self.assertNotIn("-var=production_enabled=false", applies[0])

    def test_central_recorded_delete_needs_central_approval_for_either_owner(self):
        self.write_plan("delete")
        result = self.policy(root="shared", ci=True)
        self.assertEqual(result.returncode, 1)
        self.assertIn("deploy/approvals/shared/shared.json", result.stderr)
        approval = self.approve(root="shared", commit=True)
        self.assertEqual(json.loads(approval.read_text())["environment"], "ciproxy/shared/shared")
        self.assertEqual(self.policy(root="shared", ci=True).returncode, 0)
        self.assertEqual(self.policy(environment="production", root="shared", ci=True).returncode, 0)
        # One central approval cannot authorize a delete of the separate edge root.
        result = self.policy(root="edge", ci=True)
        self.assertEqual(result.returncode, 1)
        self.assertIn("deploy/approvals/main-preview/edge.json", result.stderr)

    def test_operator_manifest_keeps_rbac_and_pod_traffic_in_one_namespace(self):
        for environment in ("main-preview", "production"):
            with self.subTest(environment=environment):
                result = self.cli("namespace_access.py", environment, "--render")
                self.assertEqual(result.returncode, 0, result.stderr)
                items = json.loads(result.stdout)["items"]
                self.assertEqual([item["kind"] for item in items], ["Namespace", "Role", "RoleBinding", "NetworkPolicy"])
                self.assertEqual(items[0]["metadata"]["name"], "ciproxy-" + environment)
                self.assertEqual(items[0]["metadata"].get("labels", {}), {
                    "pod-security.kubernetes.io/enforce": "baseline",
                    "pod-security.kubernetes.io/audit": "baseline",
                    "pod-security.kubernetes.io/warn": "baseline",
                })
                self.assertEqual(items[2]["subjects"], [{"kind": "User", "name": f"ciproxy-{environment}-deploy@ciproxy.iam.gserviceaccount.com"}])
                self.assertEqual({item["metadata"].get("namespace") for item in items[1:]}, {"ciproxy-" + environment})
                self.assertEqual(items[2]["roleRef"]["kind"], "Role")
                resources = {resource for rule in items[1]["rules"] for resource in rule["resources"]}
                self.assertEqual(resources.intersection({"*", "namespaces", "roles", "rolebindings", "clusterroles", "clusterrolebindings"}), set())
                egress = items[3]["spec"]["egress"]
                self.assertEqual(egress[0], {"to": [{"podSelector": {}}]})
                self.assertEqual(egress[1]["to"][0]["ipBlock"]["except"], ["10.84.0.0/14"])

    def test_failed_neg_wait_does_not_publish_ready_inputs(self):
        self.env.update(RELEASE_SERVER_IMAGE_DIGEST="sha256:" + "a" * 64,
                        ALERT_EMAIL="preview@example.invalid", FAKE_NEG_FAIL="true")
        result = self.cli("deploy.py", "main-preview")
        self.assertEqual(result.returncode, 1)
        self.assertFalse((self.root / ".context/deploy/main-preview/ready-inputs.json").exists())

    def test_ready_snapshot_rejects_other_commit_environment_run_or_attempt(self):
        self.trusted()
        path = self.root / ".context/deploy/main-preview/ready-inputs.json"
        path.parent.mkdir(parents=True)
        original = {"environment": "main-preview", "commit": self.source, "run": "1234", "attempt": "1",
                    "inputs": {"control_plane_enabled": False, "enable_uptime_checks": False,
                               "notification_emails": ["preview@example.invalid"],
                               "allowed_zones": ["europe-west1-b"], "source_sha": self.source}}
        for field, value in (("environment", "production"), ("commit", "0" * 40), ("run", "other"), ("attempt", "2")):
            with self.subTest(field=field):
                path.write_text(json.dumps(dict(original, **{field: value})))
                self.log.write_text("")
                result = self.cli("terraform.py", "plan", "main-preview", "foundation")
                self.assertEqual(result.returncode, 1)
                self.assertIn("Workload snapshot is stale", result.stderr)
                calls = [json.loads(line) for line in self.log.read_text().splitlines()]
                self.assertEqual([call["args"][1] for call in calls if call["tool"] == "terraform"], ["init"])

    def test_production_preview_edge_uses_exact_upstream_artifact_not_latest_state(self):
        self.trusted("production")
        path = self.snapshot(run="5678", attempt="3", control_plane_enabled=True)
        # A later unreviewed snapshot write is not an authoritative input.
        store = self.root / ".context/remote-states.json"
        store.write_text(json.dumps({"gs://ciproxy-main-preview-tfstate-ciproxy/main-preview/foundation/default.tfstate":
                                    '{"source_sha":"forged","allowed_zones":["missing-zone"]}'}))
        output = self.root / ".context/route-output"
        self.env["GITHUB_OUTPUT"] = str(output)
        result = self.cli("ci_roots.py", "production", "preview-edge")
        self.assertEqual(result.returncode, 0, result.stderr)
        selected = dict(line.split("=", 1) for line in output.read_text().splitlines())
        self.assertEqual(selected, {"roots": "edge", "target": "main-preview",
                                    "evidence_name": "workload-main-preview-3", "evidence_run": "5678"})
        result = self.cli("terraform.py", "plan", "production", "edge", "--target", "main-preview")
        self.assertEqual(result.returncode, 0, result.stderr)
        folder = self.root / ".context/plans/production/edge-main-preview"
        metadata = json.loads((folder / "metadata.json").read_text())
        self.assertEqual((metadata["environment"], metadata["target"], metadata["run"], metadata["attempt"]),
                         ("production", "main-preview", "1234", "1"))
        self.assertEqual(metadata["workload"], {"environment": "main-preview", "commit": self.source, "run": "5678", "attempt": "3"})
        inputs = json.loads((folder / "inputs.tfvars.json").read_text())
        self.assertEqual(inputs["workload"]["allowed_zones"], ["europe-west1-b"])
        self.assertTrue(inputs["workload"]["control_plane_enabled"])
        original = json.loads(path.read_text())
        for field, wrong in (("run", "9999"), ("attempt", "4"), ("commit", "f" * 40), ("environment", "production")):
            with self.subTest(field=field):
                path.write_text(json.dumps(dict(original, **{field: wrong})))
                result = self.cli("terraform.py", "plan", "production", "edge", "--target", "main-preview")
                self.assertEqual(result.returncode, 1)
                self.assertIn("Workload snapshot is stale", result.stderr)

    def test_edge_cli_preserves_targets_and_reports_preview_failure(self):
        self.trusted("production")
        self.snapshot("production")
        self.snapshot(run="5678", attempt="3")
        self.env["FAKE_FAIL_EDGE_PREFIX"] = "main-preview/edge"
        for action in ("plan", "apply"):
            result = self.cli("terraform.py", action, "production", "edge")
            self.assertEqual(result.returncode, 0, result.stderr)
        result = self.cli("deploy.py", "production", "--phase", "invalidate")
        self.assertEqual(result.returncode, 0, result.stderr)
        result = self.cli("terraform.py", "plan", "production", "edge", "--target", "main-preview")
        self.assertEqual(result.returncode, 0, result.stderr)
        result = self.cli("terraform.py", "apply", "production", "edge", "--target", "main-preview")
        self.assertEqual(result.returncode, 1)
        self.assertIn("preview networkEndpointGroups", result.stdout)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()]
        invalidation = next(i for i, call in enumerate(calls) if call["tool"] == "gcloud" and call["args"][:3] == ["compute", "url-maps", "invalidate-cdn-cache"])
        self.assertEqual(calls[invalidation]["args"][3], "ciproxy-production")

    def test_root_targets_are_bound_and_prelaunch_cannot_select_production(self):
        self.trusted()
        result = self.cli("terraform.py", "plan", "main-preview", "data", "--target", "production")
        self.assertEqual(result.returncode, 1)
        self.assertIn("Only production", result.stderr)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()]
        self.assertEqual([call for call in calls if call["tool"] == "terraform"], [])
