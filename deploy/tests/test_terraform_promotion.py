"""Saved-plan promotion, state handoff, and workload CLI regression cases."""

import json

from deploy.tests.fake_promotion_tools import install
from deploy.tests.promotion_support import PromotionCase


class TerraformPromotionTests(PromotionCase):
    def setUp(self):
        super().setUp()
        self.log = install(self)
        self.write_plan("create")

    def commands(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()]

    def prepare(self, root="seed"):
        self.trusted()
        result = self.cli("terraform.py", "plan", "main-preview", root)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.log.write_text("")
        return self.root / ".context/plans/main-preview" / root

    def test_apply_uses_the_saved_binary_and_never_replans(self):
        scratch = self.prepare()
        result = self.cli("terraform.py", "apply", "main-preview", "seed")
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = [call for call in self.commands() if call["tool"] == "terraform"]
        self.assertEqual([call["args"][1] for call in calls], ["init", "show", "apply"])
        self.assertEqual(calls[-1]["args"], [
            "-chdir=deploy/terraform/seed", "apply", "-input=false", "-lock-timeout=5m",
            str(scratch / "release.tfplan"),
        ])
        self.assertEqual((scratch / "release.tfplan").read_bytes(), b"SAVED-BINARY")

    def test_apply_rejects_changed_commit_run_attempt_configuration_and_binary(self):
        scratch = self.prepare()
        original = (scratch / "metadata.json").read_text()
        for field, value in (("commit", "0" * 40), ("run", "9999"), ("attempt", "0"),
                             ("environment", "production"), ("root", "edge"),
                             ("configurationDigest", "sha256:old"), ("planDigest", "sha256:edited")):
            with self.subTest(field=field):
                document = json.loads(original)
                document[field] = value
                (scratch / "metadata.json").write_text(json.dumps(document))
                result = self.cli("terraform.py", "apply", "main-preview", "seed")
                self.assertEqual(result.returncode, 1)
                self.assertIn("Saved plan is stale or changed", result.stderr)
                self.assertEqual(self.commands(), [])
        (scratch / "metadata.json").write_text(original)
        binary = scratch / "release.tfplan"
        binary.write_bytes(b"EDITED-BINARY")
        result = self.cli("terraform.py", "apply", "main-preview", "seed")
        self.assertEqual(result.returncode, 1)
        self.assertIn("Saved plan is stale or changed", result.stderr)
        binary.write_bytes(b"SAVED-BINARY")
        self.env["GITHUB_RUN_ATTEMPT"] = "2"
        result = self.cli("terraform.py", "apply", "main-preview", "seed")
        self.assertEqual(result.returncode, 1)
        self.assertIn("Start a new run with all jobs", result.stderr)

    def test_edited_artifact_json_cannot_hide_a_binary_delete(self):
        scratch = self.prepare("edge")
        (scratch / "plan.json").write_text('{"format_version":"1.2","resource_changes":[]}')
        self.write_plan("delete")
        result = self.cli("terraform.py", "apply", "main-preview", "edge")
        self.assertEqual(result.returncode, 1)
        self.assertIn("approval is required", result.stderr)
        self.assertEqual([call["args"][1] for call in self.commands() if call["tool"] == "terraform"], ["init", "show"])
        self.assertEqual(json.loads((scratch / "plan.json").read_text())["resource_changes"][0]
                         ["change"]["actions"], ["delete"])

    def test_apply_outside_the_reusable_workflow_stops_before_provider_calls(self):
        self.prepare()
        self.env.pop("CIPROXY_REUSABLE_APPLY")
        result = self.cli("terraform.py", "apply", "main-preview", "seed")
        self.assertEqual(result.returncode, 1)
        self.assertIn("requires the reusable apply workflow", result.stderr)
        self.assertEqual(self.commands(), [])

    def test_failed_new_plan_invalidates_the_old_saved_plan(self):
        self.prepare()
        self.env["FAKE_PLAN_STATUS"] = "1"
        result = self.cli("terraform.py", "plan", "main-preview", "seed")
        self.assertEqual(result.returncode, 1)
        self.assertIn("plan failed with status 1", result.stderr)
        self.log.write_text("")
        result = self.cli("terraform.py", "apply", "main-preview", "seed")
        self.assertEqual(result.returncode, 1)
        self.assertIn("Run a new plan", result.stderr)
        self.assertEqual(self.commands(), [])

    def test_environment_snapshot_records_its_own_runtime_and_uptime_flags(self):
        self.trusted()
        self.write_plan("output-only")
        self.env["ALERT_EMAIL"] = "preview@example.invalid"
        self.env["CONTROL_PLANE_RUNTIME_READY"] = "true"
        self.env["PUBLIC_UPTIME_ENABLED"] = "false"
        snapshot = self.root / ".context/deploy/main-preview/ready-inputs.json"
        snapshot.parent.mkdir(parents=True, exist_ok=True)
        snapshot.write_text(json.dumps({
            "environment": "main-preview", "commit": self.source, "run": "1234", "attempt": "1",
            "inputs": {"notification_emails": ["preview@example.invalid"], "control_plane_enabled": True,
                       "enable_uptime_checks": False, "allowed_zones": ["europe-west1-b"], "source_sha": self.source},
        }))
        result = self.cli("terraform.py", "plan", "main-preview", "foundation")
        self.assertEqual(result.returncode, 0, result.stderr)
        variables = self.root / ".context/plans/main-preview/foundation/inputs.tfvars.json"
        self.assertEqual(json.loads(variables.read_text()), {"notification_emails": ["preview@example.invalid"], "control_plane_enabled": True, "enable_uptime_checks": False, "allowed_zones": ["europe-west1-b"], "source_sha": self.source})

    def test_seed_creation_migrates_state_before_bootstrap_and_can_resume(self):
        result = self.cli("bootstrap.sh", "main-preview")
        self.assertEqual(result.returncode, 0, result.stderr)
        store = self.root / ".context/remote-states.json"
        self.assertEqual(set(json.loads(store.read_text())), {
            "gs://ciproxy-trust-tfstate-ciproxy/trust/seed/default.tfstate",
            "gs://ciproxy-trust-tfstate-ciproxy/trust/bootstrap/default.tfstate",
            "gs://ciproxy-shared-tfstate-ciproxy/shared/shared/default.tfstate",
            "gs://ciproxy-shared-tfstate-ciproxy/shared/edge/default.tfstate",
            "gs://ciproxy-main-preview-tfstate-ciproxy/main-preview/foundation/default.tfstate",
            "gs://ciproxy-production-tfstate-ciproxy/production/foundation/default.tfstate",
        })
        terraform = [call["args"] for call in self.commands() if call["tool"] == "terraform"]
        self.assertEqual([args[1] for args in terraform], [
            "init", "apply", "init", "init", "init", "init", "init", "init", "init", "output", "apply", "output",
        ])
        self.assertIn("-migrate-state", terraform[2])
        self.assertNotIn("-force-copy", terraform[2])
        self.assertNotIn("-input=false", terraform[2])
        self.assertIn("-backend-config=prefix=trust/seed", terraform[2])
        self.assertEqual([args[-1] for args in terraform[3:9]], [
            "-backend-config=prefix=trust/seed", "-backend-config=prefix=trust/bootstrap",
            "-backend-config=prefix=shared/shared", "-backend-config=prefix=main-preview/foundation",
            "-backend-config=prefix=shared/edge", "-backend-config=prefix=production/foundation",
        ])
        self.assertEqual(terraform[10][0], "-chdir=deploy/terraform/bootstrap")
        backup = self.root / ".context/deploy/ciproxy/seed.tfstate"
        self.assertEqual(json.loads(backup.read_text()), {"version": 4, "serial": 1})
        existing = {name: f'{{"version":4,"serial":{index + 8},"lineage":"kept"}}'
                    for index, name in enumerate(json.loads(store.read_text()))}
        store.write_text(json.dumps(existing))
        self.log.write_text("")
        result = self.cli("bootstrap.sh", "main-preview")
        self.assertEqual(result.returncode, 0, result.stderr)
        applies = [call for call in self.commands() if call["tool"] == "terraform"
                   and call["args"][1] == "apply"]
        self.assertEqual([call["args"][0] for call in applies], ["-chdir=deploy/terraform/bootstrap"])
        self.assertEqual(json.loads(store.read_text()), existing)
        self.assertNotIn("-force-copy", [arg for call in self.commands() for arg in call["args"]])

    def test_workloads_wait_for_negs_without_applying_terraform(self):
        self.env["ALERT_EMAIL"] = "preview@example.invalid"
        self.env["RELEASE_SERVER_IMAGE_DIGEST"] = "sha256:" + "a" * 64
        result = self.cli("deploy.py", "main-preview")
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = self.commands()
        self.assertEqual([call["args"][1] for call in calls if call["tool"] == "terraform"],
                         ["init", "output"])
        helm = [index for index, call in enumerate(calls) if call["tool"] == "helm"]
        neg = next(index for index, call in enumerate(calls)
                   if call["tool"] == "kubectl" and call["args"][:2] == ["get", "service"])
        self.assertGreater(neg, max(helm))
        self.assertIn("All configured NEGs are ready", result.stdout)

    def test_cdn_invalidation_is_a_separate_post_apply_command(self):
        result = self.cli("deploy.py", "main-preview", "--phase", "invalidate")
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = self.commands()
        self.assertEqual(calls[-1], {"tool": "gcloud", "args": [
            "compute", "url-maps", "invalidate-cdn-cache", "ciproxy-main-preview",
            "--path=/*", "--project=ciproxy",
        ]})
