"""Deploy fixtures for optional Secret Manager values and the rendered pod."""

import json
import shutil
import subprocess

from deploy.tests.fake_promotion_tools import install
from deploy.tests.promotion_support import PromotionCase, ROOT


class OptionalSecretCase(PromotionCase):
    def setUp(self):
        super().setUp()
        self.log = install(self)
        self.env.update({
            "ALERT_EMAIL": "deploy@example.invalid",
            "RELEASE_SERVER_IMAGE_DIGEST": "sha256:" + "a" * 64,
            "CONTROL_PLANE_RUNTIME_READY": "true",
            "CONTROL_PLANE_IMAGE": "example.invalid/control@sha256:" + "b" * 64,
        })
        chart = self.root / "deploy/helm/ciproxy"
        shutil.copytree(ROOT / "deploy/helm/ciproxy", chart)
        # Helm parses its own rendered deployment into JSON for assertions.
        (chart / "templates/probe.yaml").write_text(
            '{{ include (print $.Template.BasePath "/control-plane.yaml") . | fromYaml | toJson }}\n'
        )

    def calls(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()]

    def secrets(self, name):
        return [call["manifest"] for call in self.calls() if "manifest" in call
                and call["manifest"]["metadata"]["name"] == name]

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

    def variable(self, name):
        variables = self.pod_template()["spec"]["containers"][0]["env"]
        return next((item for item in variables if item["name"] == name), None)
