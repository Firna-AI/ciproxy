"""Dashboard staging precedes Helm, and full publication follows it."""

import json

from deploy.tests.fake_promotion_tools import install
from deploy.tests.promotion_support import PromotionCase


class DeployOrderTests(PromotionCase):
    def setUp(self):
        super().setUp()
        self.env = {
            "PATH": self.env["PATH"],
            "LC_ALL": "C",
            "TZ": "UTC",
            "ALERT_EMAIL": "deploy@example.invalid",
            "RELEASE_SERVER_IMAGE_DIGEST": "sha256:" + "a" * 64,
            "CONTROL_PLANE_RUNTIME_READY": "false",
        }
        self.log = install(self)

    def rollout_steps(self):
        steps = []
        for line in self.log.read_text().splitlines():
            call = json.loads(line)
            arguments = call["args"]
            if call["tool"] == "helm" and arguments[:1] == ["upgrade"]:
                steps.append(arguments[3:4])
            elif call["tool"] == "python3" and arguments[:1] == ["deploy/scripts/publish_static.py"]:
                steps.append(arguments[1:])
        return steps

    def test_disabled_runtime_stages_dashboard_before_helm_and_publishes_after(self):
        result = self.cli("deploy.py", "main-preview")

        self.assertEqual(result.returncode, 0, result.stderr)
        dashboard_directory = "--directory=ts/app/dist"
        self.assertEqual(self.rollout_steps(), [
            [dashboard_directory, "--bucket=app-bucket", "--new-only"],
            ["deploy/helm/cli-release-server"],
            ["deploy/helm/ciproxy"],
            ["--directory=ts/site/dist", "--bucket=site-bucket"],
            [dashboard_directory, "--bucket=app-bucket"],
        ])

    def test_stage_failure_stops_enabled_runtime_before_helm_or_full_publication(self):
        self.env.update({
            "CONTROL_PLANE_RUNTIME_READY": "true",
            "CONTROL_PLANE_IMAGE": "example.invalid/control-plane@sha256:" + "b" * 64,
            "FAKE_STAGE_FAIL": "true",
        })

        result = self.cli("deploy.py", "main-preview")

        self.assertEqual(self.rollout_steps(), [
            ["--directory=ts/app/dist", "--bucket=app-bucket", "--new-only"],
        ])
        self.assertEqual(result.returncode, 1, result.stderr)
