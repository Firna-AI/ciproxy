"""Production selects the image recorded by its exact successful preview."""

import json

from deploy.tests.fake_promotion_tools import install
from deploy.tests.image_support import IMAGE, ImageCase
from deploy.tests.promotion_support import PromotionCase


class ImagePromotionTests(ImageCase):
    def test_production_uses_exact_preview_image_and_rejects_other_evidence(self):
        self.trusted("production")
        path = self.snapshot("main-preview", run="5678", attempt="3",
                             control_plane_enabled=True, control_plane_image=IMAGE)
        self.env["CONTROL_PLANE_IMAGE"] = "ambient-stale-image"
        self.env["CONTROL_PLANE_ROLLBACK_IMAGE"] = "ambient-production-override"
        selected = self.cli("image_promotion.py", "preview-evidence")
        self.assertEqual(selected.returncode, 0, selected.stderr)
        self.assertEqual(selected.stdout, "evidence_name=workload-main-preview-3\nevidence_run=5678\n")
        result = self.cli("image_promotion.py", "promote")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "image=" + IMAGE + "\n")
        original = json.loads(path.read_text())
        for field, value in (("run", "wrong"), ("attempt", "2"), ("commit", "c" * 40),
                             ("environment", "production")):
            with self.subTest(changed_field=field):
                path.write_text(json.dumps(dict(original, **{field: value})))
                rejected = self.cli("image_promotion.py", "promote")
                self.assertEqual(rejected.returncode, 1)
                self.assertEqual(rejected.stdout, "")
        for image in ("other.invalid/image@sha256:" + "a" * 64, "moving:tag"):
            changed = json.loads(json.dumps(original))
            changed["inputs"]["control_plane_image"] = image
            path.write_text(json.dumps(changed))
            self.assertEqual(self.cli("image_promotion.py", "promote").returncode, 1)
        self.assertEqual(self.calls(), [])


class ImageSnapshotTests(PromotionCase):
    def test_workload_records_image_without_adding_a_foundation_tfvar(self):
        install(self)
        self.trusted()
        self.env.update(ALERT_EMAIL="fixture@example.invalid", RELEASE_SERVER_IMAGE_DIGEST="sha256:" + "c" * 64,
                        CONTROL_PLANE_RUNTIME_READY="true", CONTROL_PLANE_IMAGE=IMAGE)
        result = self.cli("deploy.py", "main-preview")
        self.assertEqual(result.returncode, 0, result.stderr)
        snapshot = self.root / ".context/deploy/main-preview/ready-inputs.json"
        self.assertEqual(json.loads(snapshot.read_text())["inputs"].get("control_plane_image"), IMAGE)
        self.write_plan("create")
        result = self.cli("terraform.py", "plan", "main-preview", "foundation")
        self.assertEqual(result.returncode, 0, result.stderr)
        variables = self.root / ".context/plans/main-preview/foundation/inputs.tfvars.json"
        self.assertNotIn("control_plane_image", json.loads(variables.read_text()))

    def test_disabled_preview_records_no_image_and_only_static_production_can_follow(self):
        log = install(self)
        self.trusted()
        self.env.update(ALERT_EMAIL="fixture@example.invalid", RELEASE_SERVER_IMAGE_DIGEST="sha256:" + "c" * 64,
                        CONTROL_PLANE_RUNTIME_READY="false", CONTROL_PLANE_IMAGE=IMAGE)
        deployed = self.cli("deploy.py", "main-preview")
        self.assertEqual(deployed.returncode, 0, deployed.stderr)
        snapshot = self.root / ".context/deploy/main-preview/ready-inputs.json"
        record = json.loads(snapshot.read_text())
        self.assertEqual(record["inputs"]["control_plane_image"], "")
        self.assertFalse(record["inputs"]["control_plane_enabled"])
        self.trusted("production")
        event = json.loads(self.event.read_text())
        event["workflow_run"].update(id=int(record["run"]), run_attempt=int(record["attempt"]))
        self.event.write_text(json.dumps(event))
        self.env["CONTROL_PLANE_IMAGE"] = "stale-production-override"
        selected = self.cli("image_promotion.py", "promote")
        self.assertEqual(selected.returncode, 0, selected.stderr)
        self.assertEqual(selected.stdout, "image=\n")
        self.assertIn("disabled preview runtime", selected.stderr)
        self.env.update(CONTROL_PLANE_IMAGE="", CONTROL_PLANE_RUNTIME_READY="true")
        log.write_text("")
        rejected = self.cli("deploy.py", "production")
        self.assertEqual(rejected.returncode, 1)
        self.assertIn("A ready runtime requires CONTROL_PLANE_IMAGE pinned by SHA-256", rejected.stderr)
        self.assertEqual(log.read_text(), "")
        self.env["CONTROL_PLANE_RUNTIME_READY"] = "false"
        static = self.cli("deploy.py", "production")
        self.assertEqual(static.returncode, 0, static.stderr)
        production = self.root / ".context/deploy/production/ready-inputs.json"
        self.assertEqual(json.loads(production.read_text())["inputs"]["control_plane_image"], "")

    def test_image_registry_ready_persists_through_operator_and_ci_bootstrap(self):
        log = install(self)
        for flags, expected in (([], False), (["--image-registry"], True), ([], True)):
            with self.subTest(flags=flags, registry_ready=expected):
                result = self.cli("bootstrap.sh", "main-preview", "--existing-state", *flags)
                self.assertEqual(result.returncode, 0, result.stderr)
                calls = [json.loads(line) for line in log.read_text().splitlines()]
                applies = [call for call in calls if call["tool"] == "terraform"
                           and len(call["args"]) > 1 and call["args"][1] == "apply"]
                self.assertEqual(applies[-1]["variables"].get("control_plane_registry_ready"), expected)
        self.trusted()
        self.write_plan("create")
        result = self.cli("terraform.py", "plan", "main-preview", "bootstrap")
        self.assertEqual(result.returncode, 0, result.stderr)
        variables = self.root / ".context/plans/main-preview/bootstrap/inputs.tfvars.json"
        self.assertEqual(json.loads(variables.read_text()).get("control_plane_registry_ready"), True)
