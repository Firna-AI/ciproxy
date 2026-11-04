"""Regression coverage for NEG coverage before attaching runtime backends."""

import json
import unittest
from unittest.mock import patch

from deploy.scripts import deploy_support


class NegReadinessTests(unittest.TestCase):
    def test_waits_for_all_configured_zones_before_returning(self):
        partial = {
            "metadata": {"annotations": {"cloud.google.com/neg-status": json.dumps({
                "network_endpoint_groups": {"8080": "ciproxy-main-preview-release"},
                "zones": ["europe-west1-b"],
            })}},
        }
        complete = {
            "metadata": {"annotations": {"cloud.google.com/neg-status": json.dumps({
                "network_endpoint_groups": {"8080": "ciproxy-main-preview-release"},
                "zones": ["europe-west1-d", "europe-west1-b", "europe-west1-c"],
            })}},
        }
        with (
            patch.object(deploy_support, "run", side_effect=[
                json.dumps(partial), json.dumps(complete),
            ]) as query,
            patch.object(deploy_support.time, "sleep"),
            patch.object(deploy_support.time, "monotonic", side_effect=[0, 1, 2]),
        ):
            zones = deploy_support.neg_zones(
                "release-neg", "ciproxy-main-preview-release",
                ["europe-west1-b", "europe-west1-c", "europe-west1-d"],
            )
        self.assertEqual(zones, ["europe-west1-b", "europe-west1-c", "europe-west1-d"])
        self.assertEqual(query.call_count, 2)


if __name__ == "__main__":
    unittest.main()
