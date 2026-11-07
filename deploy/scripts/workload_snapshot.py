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
        "control_plane_image": os.environ.get("CONTROL_PLANE_IMAGE", ""),
    }
    path = location(environment)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps({**binding(environment), "inputs": inputs}, indent=2) + "\n")


def expected_binding(environment, caller=None):
    """An upstream artifact is identified by the successful webhook run/attempt."""
    caller = caller or environment
    expected = binding(environment)
    if caller != environment:
        if caller != "production" or environment != "main-preview":
            raise ValueError("Only production can select upstream preview evidence.")
        event = load_json(Path(os.environ["GITHUB_EVENT_PATH"]))
        run = event["workflow_run"]
        if run["head_sha"] != expected["commit"] or not run.get("id") or not run.get("run_attempt"):
            raise ValueError("Successful preview run, attempt, and checkout must match.")
        expected.update(run=str(run["id"]), attempt=str(run["run_attempt"]))
    return expected


def evidence(environment, caller=None):
    expected = expected_binding(environment, caller)
    return {"evidence_name": f"workload-{environment}-{expected['attempt']}",
            "evidence_run": expected["run"]}


def load(environment, caller=None):
    document = load_json(location(environment))
    if {key: value for key, value in document.items() if key != "inputs"} != expected_binding(environment, caller):
        raise ValueError("Workload snapshot is stale or from another environment/run. Deploy again.")
    return document["inputs"]
