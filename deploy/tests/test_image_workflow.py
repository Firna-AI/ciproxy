"""Image credential and archive boundaries in the real workflow graph."""

import base64
import contextlib
import io
import json
import os
from pathlib import Path
import shlex
import tempfile
import unittest
from unittest.mock import patch

from deploy.tests import image_security_support as security


class ImageWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.documents = security.workflows()
        cls.providers = security.providers()

    def test_only_primary_reusable_image_jobs_satisfy_real_provider_conditions(self):
        workload = self.documents["deploy-main-preview.yml"]["jobs"]["workloads"]
        ordinary = security.claims(workload)
        self.assertNotIn("job_workflow_ref", ordinary)
        self.assertEqual(set(self.providers), {"image_lookup", "image_publish"})
        for name, provider in self.providers.items():
            with self.subTest(identity=name, scenario="ordinary workload"):
                self.assertFalse(security.admits(provider, ordinary))
                self.assertEqual(provider["attribute_mapping"]["attribute.job_workflow_ref"], "assertion.job_workflow_ref")
            for scenario, reference in security.UNTRUSTED.items():
                with self.subTest(identity=name, scenario=scenario):
                    self.assertFalse(security.admits(provider, dict(ordinary, job_workflow_ref=reference)))
        privileged = security.image_jobs(self.documents)
        self.assertEqual(len(privileged), 2)
        for job_name, job, _, _ in privileged:
            for name, provider in self.providers.items():
                with self.subTest(identity=name, job=job_name, scenario="trusted primary workflow"):
                    self.assertTrue(security.admits(provider, security.claims(job, security.TRUSTED)))
        build = next(item for item in security.jobs(self.documents) if any(
            "control_plane_image.py build" in step.get("run", "") for step in item[1].get("steps", [])))
        permissions = build[1].get("permissions", build[3])
        self.assertEqual(permissions, {"contents": "read"})
        self.assertNotIn("environment", build[1])
        for name, provider in self.providers.items():
            with self.subTest(identity=name, scenario="source build without OIDC"):
                self.assertFalse(security.admits(provider, security.claims(build[1], build[2]),
                                                  oidc=permissions.get("id-token") == "write"))

    def test_primary_tools_use_the_reusable_jobs_own_revision_without_token_output(self):
        privileged = security.image_jobs(self.documents)
        self.assertEqual(len(privileged), 2)
        for name, _, origin, _ in privileged:
            with self.subTest(job=name, scenario="preview calls primary workflow"):
                self.assertEqual(origin, security.TRUSTED)
        for _, job, _, _ in privileged:
            for executable in job.get("steps", []):
                command = executable.get("run", "")
                if not command or command.startswith("python3 - <<"):
                    continue
                arguments = shlex.split(command)
                self.assertIn(arguments[0], ("python3", "bash"))
                self.assertTrue(arguments[1].startswith("deploy/scripts/"), arguments)
        lookup = next(job for _, job, _, _ in privileged if job["steps"][0].get("run"))
        step = lookup["steps"][0]
        lines = step["run"].splitlines()
        code = "\n".join(lines[1:-1])
        scratch = security.ROOT / ".context/control-plane-image/item-100-primary/tests"
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=scratch) as directory:
            environment = {"ACTIONS_ID_TOKEN_REQUEST_URL": "https://fixture.invalid/id?request=1",
                           "ACTIONS_ID_TOKEN_REQUEST_TOKEN": "fixture-request-credential",
                           "GITHUB_SHA": security.SOURCE_SHA}
            scenarios = [("trusted primary workflow", security.TRUSTED, security.TOOLS_SHA, 0)]
            scenarios.extend((scenario, reference, security.TOOLS_SHA, 1)
                             for scenario, reference in security.UNTRUSTED.items())
            scenarios.append(("invalid tools revision", security.TRUSTED, "not-a-commit", 1))
            for index, (scenario, reference, revision, status) in enumerate(scenarios):
                with self.subTest(scenario=scenario):
                    output = Path(directory) / str(index)
                    environment["GITHUB_OUTPUT"] = str(output)
                    payload = base64.urlsafe_b64encode(json.dumps({"job_workflow_ref": reference,
                                                                 "job_workflow_sha": revision}).encode()).decode()
                    token = "fixture." + payload + ".fixture-signature"
                    response = io.BytesIO(json.dumps({"value": token}).encode())
                    stdout, stderr = io.StringIO(), io.StringIO()
                    with patch.dict(os.environ, environment), patch("urllib.request.urlopen", return_value=response):
                        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                            try:
                                exec(compile(code, "<trusted workflow first step>", "exec"), {})
                                actual = 0
                            except SystemExit as error:
                                actual = error.code
                    emitted = output.read_text() if output.exists() else ""
                    self.assertEqual(actual, status)
                    self.assertEqual(emitted, "tools_sha=" + security.TOOLS_SHA + "\n" if status == 0 else "")
                    self.assertEqual(stdout.getvalue(), "")
                    self.assertNotIn(token, stdout.getvalue() + stderr.getvalue() + emitted)
                    self.assertNotIn(payload, stdout.getvalue() + stderr.getvalue() + emitted)
                    self.assertNotIn("fixture-request-credential", stdout.getvalue() + stderr.getvalue())
        for _, job, _, _ in security.jobs(self.documents):
            checkouts = [step for step in job.get("steps", []) if step.get("uses", "").startswith("actions/checkout@")]
            if not any("control_plane_image.py" in step.get("run", "") for step in job.get("steps", [])):
                continue
            trusted = checkouts[0]
            self.assertEqual(trusted["with"].get("repository"), "Firna-AI/ciproxy")
            context = {"steps": {step["id"]: {"outputs": {"tools_sha": security.TOOLS_SHA}}},
                       "needs": {"lookup": {"outputs": {"tools_sha": security.TOOLS_SHA}}},
                       "github": {"sha": security.SOURCE_SHA}}
            self.assertEqual(security.resolve(trusted["with"]["ref"], context), security.TOOLS_SHA)
            self.assertFalse(any(step.get("working-directory") == "source" for step in job.get("steps", [])))

    def test_publication_consumes_build_artifact_id_and_canonical_digest(self):
        candidates = security.jobs(self.documents)
        build = next(job for _, job, _, _ in candidates if any(
            "control_plane_image.py build" in step.get("run", "") for step in job.get("steps", [])))
        publish = next(job for _, job, _, _ in security.image_jobs(self.documents) if any(
            "control_plane_image.py publish" in step.get("run", "") for step in job.get("steps", [])))
        upload = next(step for step in build["steps"] if step.get("uses", "").startswith("actions/upload-artifact@"))
        digest = next(step for step in build["steps"] if "control_plane_image.py build" in step.get("run", ""))
        values = {"steps": {upload.get("id", ""): {"outputs": {"artifact-id": "8472"}},
                            digest.get("id", ""): {"outputs": {"digest": "sha256:" + "a" * 64}}}}
        build_outputs = {key: security.resolve(value, values) for key, value in build.get("outputs", {}).items()}
        download = next(step for step in publish["steps"] if step.get("uses", "").startswith("actions/download-artifact@"))
        context = {"needs": {"build": {"outputs": build_outputs}}}
        self.assertEqual(security.resolve(download["with"].get("artifact-ids"), context), "8472")
        self.assertNotIn("name", download["with"])
        command = next(step for step in publish["steps"] if "control_plane_image.py publish" in step.get("run", ""))
        self.assertEqual(security.resolve(command.get("env", {}).get("EXPECTED_DIGEST"), context), "sha256:" + "a" * 64)
        self.assertIn('--expected-digest "$EXPECTED_DIGEST"', command["run"])
