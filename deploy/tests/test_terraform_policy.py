"""Hand-worked destructive-policy cases through the public CLI."""

import json

from deploy.tests.promotion_support import PromotionCase


class TerraformPolicyTests(PromotionCase):
    def test_recorded_creates_and_updates_need_no_approval(self):
        for recording in ("create", "update"):
            with self.subTest(recording=recording):
                self.write_plan(recording)
                result = self.policy(ci=True)
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_recorded_output_only_plan_passes_snapshot_policy(self):
        self.write_plan("output-only")
        result = self.policy(root="foundation", ci=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_each_destructive_action_needs_an_exact_approval(self):
        plan = self.write_plan("delete")
        for actions in (["delete"], ["delete", "create"], ["create", "delete"], ["forget"]):
            with self.subTest(actions=actions):
                plan["resource_changes"][0]["change"]["actions"] = actions
                self.plan.write_text(json.dumps(plan))
                result = self.policy(ci=True)
                self.assertEqual(result.returncode, 1)
                self.assertIn("approval is required", result.stderr)

    def test_exact_committed_delete_and_replacements_pass(self):
        plan = self.write_plan("delete")
        for actions in (["delete"], ["delete", "create"], ["create", "delete"], ["forget"]):
            with self.subTest(actions=actions):
                plan["resource_changes"][0]["change"]["actions"] = actions
                self.plan.write_text(json.dumps(plan))
                approval = self.approve(commit=True)
                document = json.loads(approval.read_text())
                self.assertEqual(document["environment"], "ciproxy/shared/edge")
                self.assertEqual(document["actions"], [{
                    "address": "terraform_data.example", "actions": actions,
                }])
                result = self.policy(ci=True)
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_scope_environment_root_actions_and_reason_must_match(self):
        path = self.approve()
        original = json.loads(path.read_text())
        cases = {
            "scope": {"environment": "other/main-preview/edge"},
            "environment": {"environment": "ciproxy/production/edge"},
            "root": {"environment": "ciproxy/main-preview/foundation"},
            "digest": {"configurationDigest": "sha256:" + "0" * 64},
            "extra action": {"actions": original["actions"] * 2},
            "missing action": {"actions": []},
            "replacement order": {"actions": [{"address": "terraform_data.example",
                                                 "actions": ["create", "delete"]}]},
            "short reason": {"reason": " "},
            "long reason": {"reason": "a" * 501},
            "unknown field": {"extra": True},
            "boolean schema": {"schemaVersion": True},
        }
        for name, changes in cases.items():
            with self.subTest(case=name):
                path.write_text(json.dumps(dict(original, **changes)))
                self.assertEqual(self.policy().returncode, 1)

    def test_full_configuration_digest_includes_modules_and_locks(self):
        path = self.approve()
        for name in ("deploy/terraform/modules/foundation/main.tf",
                     "deploy/terraform/shared/.terraform.lock.hcl"):
            with self.subTest(source=name):
                source = self.root / name
                original = source.read_text()
                source.write_text(original + "# changed\n")
                result = self.policy()
                self.assertEqual(result.returncode, 1)
                self.assertIn("Terraform configuration", result.stderr)
                source.write_text(original)
        self.assertTrue(path.is_file())

    def test_stale_approval_and_duplicate_fields_fail(self):
        path = self.approve()
        original = path.read_text()
        self.write_plan("update")
        result = self.policy()
        self.assertEqual(result.returncode, 1)
        self.assertIn("stale", result.stderr)
        self.write_plan("delete")
        path.write_text(original.replace('"schemaVersion": 1', '"schemaVersion": 1, "schemaVersion": 1'))
        result = self.policy()
        self.assertEqual(result.returncode, 1)
        self.assertIn("repeats field", result.stderr)

    def test_approval_must_be_committed_and_byte_unchanged(self):
        path = self.approve()
        result = self.policy(ci=True)
        self.assertEqual(result.returncode, 1)
        self.assertIn("must be committed", result.stderr)
        self.commit()
        path.write_text(path.read_text() + "\n")
        result = self.policy(ci=True)
        self.assertEqual(result.returncode, 1)
        self.assertIn("must be unmodified", result.stderr)

    def test_approved_pool_replacement_and_all_trust_removals_stop_ci(self):
        resources = [
            ("bootstrap", "google_iam_workload_identity_pool", "github"),
            ("bootstrap", "google_iam_workload_identity_pool_provider", "github"),
            ("bootstrap", "google_service_account", 'promotion["apply"]'),
            ("bootstrap", "google_service_account_iam_member", 'promotion["plan"]'),
            ("bootstrap", "google_project_iam_member", 'apply["roles/storage.admin"]'),
            ("bootstrap", "google_project_iam_custom_role", "plan_lock"),
            ("bootstrap", "google_storage_bucket_iam_member", "plan_lock"),
            ("seed", "google_storage_bucket", "state"),
            ("seed", "google_project_service", 'identity["iam.googleapis.com"]'),
        ]
        for root, resource_type, name in resources:
            with self.subTest(root=root, resource=resource_type):
                if resource_type == "google_iam_workload_identity_pool":
                    self.write_plan("pool-replace")
                else:
                    self.write_plan(resources=[{
                        "address": f"{resource_type}.{name}", "type": resource_type,
                        "change": {"actions": ["delete", "create"]},
                    }])
                self.approve(root=root, commit=True)
                result = self.policy(root=root, ci=True)
                self.assertEqual(result.returncode, 1)
                self.assertIn("CI never deletes, replaces or forgets trust resource", result.stderr)
                result = self.policy(root=root, ci=False)
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_in_place_trust_updates_pass_and_deposed_objects_stay_distinct(self):
        self.write_plan(resources=[{
            "address": "google_iam_workload_identity_pool.github",
            "type": "google_iam_workload_identity_pool", "change": {"actions": ["update"]},
        }])
        self.assertEqual(self.policy(root="bootstrap", ci=True).returncode, 0)
        self.write_plan(resources=[{
            "address": 'google_project_service.identity["iam.googleapis.com"]',
            "type": "google_project_service", "change": {"actions": ["update"]},
        }])
        self.assertEqual(self.policy(root="seed", ci=True).returncode, 0)
        self.write_plan(resources=[
            {"address": "terraform_data.example", "change": {"actions": ["delete"]}},
            {"address": "terraform_data.example", "deposed": "deadbeef",
             "change": {"actions": ["delete"]}},
        ])
        path = self.approve(commit=True)
        self.assertEqual(json.loads(path.read_text())["actions"], [
            {"address": "terraform_data.example", "actions": ["delete"]},
            {"address": "terraform_data.example", "actions": ["delete"], "deposed": "deadbeef"},
        ])
        self.assertEqual(self.policy(ci=True).returncode, 0)
        document = json.loads(path.read_text())
        document["actions"][1]["deposed"] = "badc0ffe"
        path.write_text(json.dumps(document))
        self.assertEqual(self.policy().returncode, 1)

    def test_unknown_plan_formats_actions_and_duplicate_resources_fail_closed(self):
        original = self.write_plan("delete")
        cases = [{"format_version": "1.2"}, dict(original, resource_changes=None),
                 dict(original, resource_changes={}), dict(original, format_version="2.0"),
                 dict(original, resource_changes=[{"address": "test.old",
                                                   "change": {"actions": ["destroy"]}}]),
                 dict(original, resource_changes=original["resource_changes"] * 2)]
        for plan in cases:
            with self.subTest(plan=plan):
                self.plan.write_text(json.dumps(plan))
                self.assertEqual(self.policy().returncode, 1)
