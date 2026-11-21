"""Runtime configuration reaches Helm through the deployment command."""

import json

from deploy.tests.fake_promotion_tools import install
from deploy.tests.promotion_support import PromotionCase


class RuntimeConfigurationTests(PromotionCase):
    def test_workload_helm_uses_environment_settings_and_trusted_release_identity(self):
        log = install(self)
        self.trusted("production")
        self.env.update({
            "ALERT_EMAIL": "runtime@example.invalid",
            "RELEASE_SERVER_IMAGE_DIGEST": "sha256:" + "a" * 64,
            "CONTROL_PLANE_RUNTIME_READY": "true",
            "CONTROL_PLANE_IMAGE": "example.invalid/control-plane@sha256:" + "b" * 64,
            "CDN_SIGNING_KEY_NAME": "ciproxy-production-images-v3",
            "CIPROXY_APP_ID": "123456",
            "CIPROXY_APP_CLIENT_ID": "Iv1.runtime-fixture",
            "CIPROXY_APP_SLUG": "runtime-fixture-app",
            "GITHUB_APP_ID": "ambient-old-app-id",
            "GITHUB_OAUTH_CLIENT_ID": "ambient-old-client-id",
            "GITHUB_APP_SLUG": "ambient-old-app-slug",
            "JOB_IMAGE_MANIFEST_PATH": "release-20261106/manifest.json",
            "GITHUB_SHA": "c" * 40,
            "BUILD_ID": "untrusted-build-override",
            "DOWNLOAD_URL": "https://untrusted-download.invalid",
        })
        trusted = self.cli("promotion_event.py", "production", self.source)
        self.assertEqual(trusted.returncode, 0, trusted.stderr)
        result = self.cli("deploy.py", "production")
        self.assertEqual(result.returncode, 0, result.stderr)
        commands = [json.loads(line) for line in log.read_text().splitlines()]
        helm = next(call for call in commands
                    if call["tool"] == "helm" and "deploy/helm/ciproxy" in call["args"])
        values_path = next(argument.split("=", 1)[1] for argument in helm["args"]
                           if argument.startswith("--values="))
        values = json.loads((self.root / values_path).read_text())
        self.assertEqual(values["controlPlane"].get("env", []), [
            {"name": "GITHUB_APP_ID", "value": "123456"},
            {"name": "GITHUB_OAUTH_CLIENT_ID", "value": "Iv1.runtime-fixture"},
            {"name": "GITHUB_APP_SLUG", "value": "runtime-fixture-app"},
            {"name": "JOB_IMAGE_MANIFEST_PATH", "value": "release-20261106/manifest.json"},
            {"name": "DOWNLOAD_URL", "value": "https://downloads.invalid"},
            {"name": "BUILD_ID", "value": self.source},
        ])
        for name in ("CIPROXY_APP_ID", "CIPROXY_APP_CLIENT_ID", "CIPROXY_APP_SLUG"):
            self.env.pop(name)
        missing_inputs = self.cli("deploy.py", "production")
        self.assertEqual(missing_inputs.returncode, 0, missing_inputs.stderr)
        values = json.loads((self.root / values_path).read_text())
        self.assertEqual(values["controlPlane"]["env"], [
            {"name": "GITHUB_APP_ID", "value": ""},
            {"name": "GITHUB_OAUTH_CLIENT_ID", "value": ""},
            {"name": "GITHUB_APP_SLUG", "value": ""},
            {"name": "JOB_IMAGE_MANIFEST_PATH", "value": "release-20261106/manifest.json"},
            {"name": "DOWNLOAD_URL", "value": "https://downloads.invalid"},
            {"name": "BUILD_ID", "value": self.source},
        ])
