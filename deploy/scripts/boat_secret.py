"""Deliver the optional Boat snapshot without exposing its payload."""

import base64
import json
import re
import subprocess


def sync_boat_secret(config):
    # Capture both streams. Provider errors can echo a secret-bearing request.
    read = subprocess.run([
        "gcloud", "secrets", "versions", "access", "latest",
        f"--secret={config['boat_shared_snapshot_secret']}",
        f"--project={config['project_id']}",
    ], text=True, capture_output=True)
    missing = re.search(
        r"^ERROR: \(gcloud\.secrets\.versions\.access\) NOT_FOUND:",
        read.stderr, re.MULTILINE,
    )
    if read.returncode and not missing:
        raise RuntimeError("Boat snapshot secret read failed. Check Secret Manager access and version state.")
    snapshot = read.stdout.strip() if read.returncode == 0 else ""
    secret = {
        "apiVersion": "v1", "kind": "Secret",
        "metadata": {"name": "boat-shared-snapshot", "namespace": config["namespace"]},
        "type": "Opaque",
        # Empty data removes a previously owned key when no version exists.
        "data": {"BOAT_SHARED_SNAPSHOT": base64.b64encode(snapshot.encode()).decode()}
        if snapshot else {},
    }
    applied = subprocess.run([
        "kubectl", "apply", "--server-side", "--field-manager=ciproxy-deploy", "-f", "-",
        "-o", "jsonpath={.metadata.resourceVersion}",
    ], input=json.dumps(secret), text=True, capture_output=True)
    if applied.returncode:
        raise RuntimeError("Boat snapshot secret apply failed. Check namespace Secret access.")
    # Only public metadata reaches Helm. The checksum never hashes the payload.
    return applied.stdout.strip()
