"""Inspect ruleset writes through a fake GitHub CLI. Never contact GitHub."""

import json

from deploy.tests.fake_promotion_tools import install
from deploy.tests.promotion_support import PromotionCase


class GithubRulesetTests(PromotionCase):
    def test_ruleset_requires_pr_and_gate_and_updates_the_same_rule(self):
        log = install(self)
        self.env["GH_TOKEN"] = "fake-token-must-not-appear"
        first = self.cli("github-ruleset.sh")
        self.assertEqual(first.returncode, 0, first.stderr)
        second = self.cli("github-ruleset.sh")
        self.assertEqual(second.returncode, 0, second.stderr)
        calls = [json.loads(line) for line in log.read_text().splitlines()]
        writes = [call for call in calls if "payload" in call]
        self.assertEqual([call["args"][:4] for call in writes], [
            ["api", "--method", "POST", "repos/Firna-AI/ciproxy/rulesets"],
            ["api", "--method", "PUT", "repos/Firna-AI/ciproxy/rulesets/42"],
        ])
        self.assertEqual(writes[0]["payload"], writes[1]["payload"])
        payload = writes[0]["payload"]
        self.assertEqual(payload["conditions"], {
            "ref_name": {"include": ["refs/heads/main"], "exclude": []},
        })
        self.assertEqual(payload["enforcement"], "active")
        self.assertEqual(payload["bypass_actors"], [])
        rules = payload["rules"]
        self.assertEqual([rule["type"] for rule in rules], [
            "deletion", "non_fast_forward", "pull_request", "required_status_checks",
        ])
        self.assertEqual(rules[2]["parameters"]["required_approving_review_count"], 0)
        self.assertEqual(rules[3]["parameters"], {
            "required_status_checks": [{"context": "Repository gate", "integration_id": 15368}],
            "strict_required_status_checks_policy": False,
        })
        self.assertNotIn(self.env["GH_TOKEN"], first.stdout + first.stderr + second.stdout + second.stderr)
