"""Release channels and the upstream revision reach the deployment values."""

import json
import re

from deploy.tests.fake_promotion_tools import install
from deploy.tests.promotion_support import PromotionCase, ROOT


class ReleaseServerTests(PromotionCase):
    def setUp(self):
        super().setUp()
        self.log = install(self)
        self.digest = "sha256:" + "a" * 64
        self.env.update({
            "ALERT_EMAIL": "release@example.invalid",
            "RELEASE_SERVER_IMAGE_DIGEST": self.digest,
            "CONTROL_PLANE_RUNTIME_READY": "false",
            "CLI_RELEASE_CHANNEL": "prerelease",
        })

    def release_values(self, environment):
        result = self.cli("deploy.py", environment)
        self.assertEqual(result.returncode, 0, result.stderr)
        commands = [json.loads(line) for line in self.log.read_text().splitlines()]
        helm = [call for call in commands if call["tool"] == "helm"
                and "deploy/helm/cli-release-server" in call["args"]][-1]
        values_path = [argument.split("=", 1)[1] for argument in helm["args"]
                       if argument.startswith("--values=")][-1]
        return json.loads((self.root / values_path).read_text())

    def test_deploy_serves_prereleases_only_in_main_preview(self):
        for environment, channel in (("main-preview", "prerelease"), ("production", "stable")):
            with self.subTest(environment=environment):
                values = self.release_values(environment)
                self.assertEqual(values["config"].get("releaseChannel"), channel)

    def test_mirror_and_deployed_image_match_the_vendored_chart_revision(self):
        chart = ROOT / "deploy/helm/cli-release-server/PROVENANCE.json"
        revision = json.loads(chart.read_text())["revision"]
        script = (ROOT / "deploy/scripts/mirror-release-image.sh").read_text()
        full = re.search(r"^full_revision=([0-9a-f]{40})$", script, re.M).group(1)
        short = re.search(r"^revision=([0-9a-f]{12})$", script, re.M).group(1)
        self.assertEqual(full, revision)
        self.assertEqual(short, revision[:12])
        values = self.release_values("main-preview")
        self.assertEqual(values["image"]["tag"], f"upstream-{short}@{self.digest}")
