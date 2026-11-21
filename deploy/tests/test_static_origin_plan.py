"""Bounded inventory of the actual mocked static-origin Terraform plans."""

import json
from pathlib import Path
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[2]


class StaticOriginPlanTests(unittest.TestCase):
    def test_every_declared_bucket_iam_grant_excludes_public_principals(self):
        init = subprocess.run(
            ["terraform", "-chdir=deploy/terraform/data", "init", "-backend=false",
             "-input=false", "-lockfile=readonly"],
            cwd=ROOT, capture_output=True, text=True,
        )
        self.assertEqual(init.returncode, 0, init.stdout + init.stderr)
        result = subprocess.run(
            ["terraform", "-chdir=deploy/terraform/data", "test", "-json", "-verbose",
             "-filter=private-origins.tftest.hcl"],
            cwd=ROOT, capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        events = [json.loads(line) for line in result.stdout.splitlines()]
        plans = [event["test_plan"] for event in events if event["type"] == "test_plan"]
        self.assertEqual(len(plans), 2, "Both preview and production plans must be inventoried.")
        grants = []
        for plan in plans:
            for resource in plan["resource_changes"]:
                kind = resource.get("type")
                after = resource["change"].get("after") or {}
                if kind == "google_storage_bucket_iam_member":
                    grants.append((resource["address"], [after.get("member")]))
                elif kind == "google_storage_bucket_iam_binding":
                    grants.append((resource["address"], after.get("members", [])))
                elif kind == "google_storage_bucket_iam_policy":
                    bindings = json.loads(after["policy_data"])["bindings"]
                    grants.append((resource["address"], [member for binding in bindings for member in binding["members"]]))
        self.assertGreaterEqual(len(grants), 4, "Actual bucket IAM resources must be present.")
        public_grants = [(address, member) for address, members in grants for member in members
                         if member in ("allUsers", "allAuthenticatedUsers")]
        self.assertEqual(public_grants, [], "No declared bucket IAM resource may grant anonymous GCS access.")
