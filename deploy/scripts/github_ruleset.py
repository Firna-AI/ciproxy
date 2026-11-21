"""Build the repository ruleset for main without reading any credential."""

import json
from pathlib import Path
import sys


NAME = "main-pull-request-and-gate"


def main(existing, destination):
    document = {
        "name": NAME,
        "target": "branch",
        "enforcement": "active",
        "bypass_actors": [],
        "conditions": {"ref_name": {"include": ["refs/heads/main"], "exclude": []}},
        "rules": [
            {"type": "deletion"},
            {"type": "non_fast_forward"},
            {"type": "pull_request", "parameters": {
                "required_approving_review_count": 0,
                "dismiss_stale_reviews_on_push": False,
                "require_code_owner_review": False,
                "require_last_push_approval": False,
                "required_review_thread_resolution": False,
            }},
            {"type": "required_status_checks", "parameters": {
                "required_status_checks": [{"context": "Repository gate", "integration_id": 15368}],
                "strict_required_status_checks_policy": False,
            }},
        ],
    }
    destination.write_text(json.dumps(document, indent=2) + "\n")
    for page in json.loads(existing.read_text()):
        for ruleset in page:
            if ruleset.get("name") == NAME and ruleset.get("target") == "branch":
                print(ruleset["id"])
                return


if __name__ == "__main__":
    main(Path(sys.argv[1]), Path(sys.argv[2]))
