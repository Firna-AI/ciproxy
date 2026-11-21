"""Production follows the exact successful main preview commit."""

import json

from deploy.tests.promotion_support import PromotionCase


class PromotionEventTests(PromotionCase):
    def test_preview_accepts_primary_and_main_pushes(self):
        self.trusted()
        for ref in ("refs/heads/calummoore/ci-proxy-load-balancer", "refs/heads/main"):
            with self.subTest(ref=ref):
                self.env["GITHUB_REF"] = ref
                self.env["GITHUB_WORKFLOW_REF"] = "Firna-AI/ciproxy/.github/workflows/deploy-main-preview.yml@" + ref
                result = self.cli("promotion_event.py", "main-preview", self.source)
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_production_deploys_the_upstream_commit_even_when_default_head_moves(self):
        self.trusted("production")
        self.env["GITHUB_SHA"] = "f" * 40
        result = self.cli("promotion_event.py", "production", self.source)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(self.source, result.stdout)

    def test_production_rejects_wrong_run_repository_branch_event_and_commit(self):
        self.trusted("production")
        original = json.loads(self.event.read_text())
        changes = {
            "name": "Untrusted preview", "status": "in_progress", "conclusion": "failure",
            "event": "pull_request", "head_branch": "other", "head_sha": "a" * 40,
            "path": ".github/workflows/other.yml",
            "repository": {"full_name": "other/ciproxy", "id": 1442045837},
            "head_repository": {"full_name": "Firna-AI/ciproxy", "id": 999},
        }
        for field, value in changes.items():
            with self.subTest(field=field):
                event = json.loads(json.dumps(original))
                event["workflow_run"][field] = value
                self.event.write_text(json.dumps(event))
                self.assertEqual(self.cli("promotion_event.py", "production", self.source).returncode, 1)
        self.event.write_text(json.dumps(original))
        for field, value in (("PRODUCTION_DEPLOY_ENABLED", "false"), ("GITHUB_REF", "refs/heads/other"),
                             ("GITHUB_EVENT_NAME", "push"), ("GITHUB_REPOSITORY", "fork/ciproxy"),
                             ("GITHUB_WORKFLOW_REF", "Firna-AI/ciproxy/.github/workflows/other.yml@refs/heads/main")):
            with self.subTest(field=field):
                result = self.cli("promotion_event.py", "production", self.source,
                                  env=dict(self.env, **{field: value}))
                self.assertEqual(result.returncode, 1)

    def test_checkout_and_preview_push_commit_must_match(self):
        self.trusted()
        result = self.cli("promotion_event.py", "main-preview", "a" * 40)
        self.assertEqual(result.returncode, 1)
        self.assertIn("must deploy the push commit", result.stderr)
        self.trusted("production")
        event = json.loads(self.event.read_text())
        event["workflow_run"]["head_sha"] = "b" * 40
        self.event.write_text(json.dumps(event))
        result = self.cli("promotion_event.py", "production", "b" * 40)
        self.assertEqual(result.returncode, 1)
        self.assertIn("Checkout does not match", result.stderr)
