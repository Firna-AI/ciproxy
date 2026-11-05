"""CLI fixtures for deployment promotion tests. No cloud command runs."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

from deploy.scripts.terraform_roots import approval_path


ROOT = Path(__file__).resolve().parents[2]
FIXTURES = ROOT / "deploy/tests/fixtures"


class PromotionCase(unittest.TestCase):
    def setUp(self):
        scratch = ROOT / ".context/terraform-promotion/tests"
        scratch.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        shutil.copytree(ROOT / "deploy/scripts", self.root / "deploy/scripts")
        for name in ("terraform/seed", "terraform/bootstrap", "terraform/shared", "terraform/shared-edge", "environments/main-preview",
                     "environments/production", "terraform/modules/foundation"):
            location = self.root / "deploy" / name
            location.mkdir(parents=True, exist_ok=True)
            (location / "main.tf").write_text('terraform {\n  backend "gcs" {}\n}\n')
            (location / ".terraform.lock.hcl").write_text("provider lock\n")
        self.env = dict(os.environ, GITHUB_ACTIONS="false")
        self.env.pop("GITHUB_EVENT_PATH", None)
        self.commit()
        self.source = self.git("rev-parse", "HEAD").strip()
        self.plan = self.root / ".context/input.json"
        self.plan.parent.mkdir(exist_ok=True)
        self.write_plan("delete")

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, text=True,
                                       stderr=subprocess.DEVNULL)

    def commit(self):
        if not (self.root / ".git").exists():
            self.git("init", "-q")
            self.git("config", "user.email", "fixture@example.invalid")
            self.git("config", "user.name", "Fixture")
            (self.root / ".gitignore").write_text(".context/\n__pycache__/\n")
        self.git("add", "deploy", ".gitignore")
        self.git("commit", "-qm", "fixture")

    def cli(self, name, *args, env=None):
        command = (["bash"] if name.endswith(".sh") else [sys.executable])
        return subprocess.run(
            [*command, f"deploy/scripts/{name}", *args], cwd=self.root,
            env=self.env if env is None else env, text=True, capture_output=True,
        )

    def write_plan(self, name=None, resources=None):
        document = (json.loads((FIXTURES / f"{name}.json").read_text()) if name else
                    {"format_version": "1.2", "resource_changes": resources})
        self.plan.write_text(json.dumps(document))
        return document

    def approve(self, environment="main-preview", root="edge", commit=False):
        result = self.cli("terraform_policy.py", "approval", environment, root,
                          str(self.plan), "--reason", "Remove the retired test resource.")
        self.assertEqual(result.returncode, 0, result.stderr)
        path = approval_path(self.root, environment, root)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(result.stdout)
        if commit:
            self.commit()
        return path

    def policy(self, environment="main-preview", root="edge", ci=False):
        return self.cli("terraform_policy.py", "check", environment, root, str(self.plan),
                        env=dict(self.env, GITHUB_ACTIONS=str(ci).lower()))

    def trusted(self, environment="main-preview"):
        repository = {"full_name": "Firna-AI/ciproxy", "id": 1442045837}
        event = {"repository": repository, "after": self.source}
        if environment == "production":
            event["workflow_run"] = {
                "name": "Deploy main preview", "status": "completed", "conclusion": "success",
                "event": "push", "head_branch": "main", "head_sha": self.source,
                "path": ".github/workflows/deploy-main-preview.yml",
                "repository": repository, "head_repository": repository,
            }
        self.event = self.root / ".context/event.json"
        self.event.write_text(json.dumps(event))
        self.env.update({
            "GITHUB_ACTIONS": "true", "GITHUB_REPOSITORY": "Firna-AI/ciproxy",
            "GITHUB_EVENT_PATH": str(self.event), "GITHUB_REF": "refs/heads/main",
            "GITHUB_WORKFLOW_REF": f"Firna-AI/ciproxy/.github/workflows/deploy-{environment}.yml@refs/heads/main",
            "GITHUB_SHA": self.source, "CIPROXY_SOURCE_SHA": self.source,
            "GITHUB_EVENT_NAME": "workflow_run" if environment == "production" else "push",
            "GITHUB_RUN_ID": "1234", "GITHUB_RUN_ATTEMPT": "1",
            "CIPROXY_REUSABLE_APPLY": "true", "PRODUCTION_DEPLOY_ENABLED": "true" if environment == "production" else "false",
            "FAKE_PRODUCTION_READY": "true" if environment == "production" else "false",
        })
