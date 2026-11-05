"""Bind edge inputs to a completed workload rollout and its CI run."""

import json
import os
from pathlib import Path
import subprocess

from terraform_approval import load_json


def binding(environment):
    return {
        "environment": environment,
        "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
        "run": os.environ.get("GITHUB_RUN_ID", ""),
        "attempt": os.environ.get("GITHUB_RUN_ATTEMPT", ""),
    }


def location(environment):
    return Path(".context/deploy") / environment / "ready-inputs.json"


def save(environment, configuration, runtime_enabled):
    email = os.environ.get("ALERT_EMAIL", "").strip()
    if not email:
        raise ValueError("Set the GitHub environment variable ALERT_EMAIL.")
    inputs = {
        "notification_emails": [email],
        "allowed_zones": configuration["allowed_zones"],
        "control_plane_enabled": runtime_enabled,
        "enable_uptime_checks": os.environ.get("PUBLIC_UPTIME_ENABLED") == "true",
        "source_sha": binding(environment)["commit"],
    }
    path = location(environment)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps({**binding(environment), "inputs": inputs}, indent=2) + "\n")


def load(environment):
    document = load_json(location(environment))
    if {key: value for key, value in document.items() if key != "inputs"} != binding(environment):
        raise ValueError("Workload snapshot is stale or from another environment/run. Deploy again.")
    return document["inputs"]
