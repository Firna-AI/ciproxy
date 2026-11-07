"""Supported App lifecycle and deployment steps for private binding probes."""

import json
import os
from pathlib import Path
import subprocess
from urllib.parse import quote
from uuid import UUID
import webbrowser

from overflow_smoke.support import wait


def organization(live):
    selected = str(UUID(live.config["binding_organization_id"]))
    return next(row for row in live.api("GET", "/organizations") if row["id"] == selected)


def installation(live, action):
    selected = organization(live)
    siblings = live.api("GET", "/organizations")
    if not any(row["id"] != selected["id"] and row["state"] == "active" for row in siblings):
        raise ValueError("the fixture needs an active sibling organization")
    identity = str(int(selected["installation_id"]))
    method, suffix, expected = {
        "suspend": ("PUT", "/suspended", "suspended"),
        "resume": ("DELETE", "/suspended", "active"),
        "remove": ("DELETE", "", "removed"),
    }[action]
    live.request(method, "https://api.github.com/app/installations/" + identity + suffix,
                 {"Authorization": "Bearer " + live.config["github_app_jwt"],
                  "Accept": "application/vnd.github+json",
                  "X-GitHub-Api-Version": "2026-03-10"})
    wait(lambda: organization(live)["state"] == expected, seconds=360)
    print("PASS: the dashboard observed the requested App lifecycle state")


def replace_inactive(live):
    if organization(live)["state"] not in {"suspended", "removed"}:
        raise ValueError("the selected fixture organization must be suspended or removed")
    name = live.config["binding_label"]
    if name == "ciproxy":
        raise ValueError("use a disposable custom label")
    label = next(row for row in live.api("GET", "/labels") if row["label"]["name"] == name)
    live.api("DELETE", "/labels/" + str(UUID(label["id"])))
    replacement = live.api("POST", "/labels", label["label"])
    if replacement["id"] == label["id"]:
        raise ValueError("the replacement kept the deleted label identity")
    print("PASS: replaced the custom label while the selected organization was inactive")


def reinstall(live, path, open_browser):
    previous = organization(live)
    if previous["state"] != "removed":
        raise ValueError("the fixture organization must be removed before reinstall")
    url = "https://github.com/apps/" + quote(live.config["github_app_slug"], safe="") + "/installations/new"
    descriptor = os.open(Path(path), os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    os.fchmod(descriptor, 0o600)
    with os.fdopen(descriptor, "w") as stream:
        json.dump({"install_url": url}, stream)
    if open_browser:
        webbrowser.open(url)
    print("ACTION: complete the browser installation from the URL in the private install file.", flush=True)

    def installed():
        current = organization(live)
        return (current["state"] == "active" and current["activation"]["admitted"]
                and current["installation_id"] != previous["installation_id"])

    wait(installed, seconds=900)
    print("PASS: observed a new active installation in the retained workspace")


def restart(live):
    base = ["kubectl"]
    if live.config.get("kubernetes_context"):
        base.extend(["--context", live.config["kubernetes_context"]])
    base.extend(["--namespace", live.config["kubernetes_namespace"], "rollout"])
    for operation in [["restart", "deployment/control-plane"],
                      ["status", "deployment/control-plane", "--timeout=240s"]]:
        result = subprocess.run(base + operation, capture_output=True, text=True,
                                check=False, timeout=250)
        if result.returncode:
            raise ValueError("control-plane rollout did not complete")
    print("PASS: control-plane restart and readiness rollout completed")
