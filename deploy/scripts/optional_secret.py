"""Deliver an optional Secret Manager value without exposing its payload."""

import base64
import json
import os
import re
import subprocess
from dataclasses import dataclass


@dataclass(frozen=True)
class OptionalSecret:
    """One optional value: its deployment output, its Kubernetes Secret, and its key.

    With `output_optional`, a deployment output without the parent counts as
    a parent with no version. This lets a commit deploy before the data
    apply that adds the parent reaches the environment.
    """

    output: str
    name: str
    key: str
    label: str
    output_optional: bool = False


# Boat stays strict: an empty Boat Secret stops Boat cleanup while sandboxes exist.
BOAT_SHARED_SNAPSHOT = OptionalSecret(
    "boat_shared_snapshot_secret", "boat-shared-snapshot", "BOAT_SHARED_SNAPSHOT", "Boat snapshot")
R2_IMAGES = OptionalSecret(
    "r2_images_secret", "r2-images", "R2_IMAGES_KEY", "R2 images key", output_optional=True)

# The Cloud SDK can write request and response bodies to the terminal and to
# its own log files. Both settings stay off for a read of a secret payload,
# whatever the job environment or the gcloud configuration says.
QUIET_GCLOUD = {"CLOUDSDK_CORE_LOG_HTTP": "false", "CLOUDSDK_CORE_DISABLE_FILE_LOGGING": "true"}


def sync_optional_secret(config, optional):
    if optional.output_optional and optional.output not in config:
        print(f"{optional.label}: the deployment output has no secret parent yet. "
              "The value stays off.", flush=True)
        return apply_secret(config, optional, "")
    # Capture both streams. Provider errors can echo a secret-bearing request.
    read = subprocess.run([
        "gcloud", "secrets", "versions", "access", "latest",
        f"--secret={config[optional.output]}",
        f"--project={config['project_id']}",
    ], text=True, capture_output=True, env={**os.environ, **QUIET_GCLOUD})
    missing = re.search(
        r"^ERROR: \(gcloud\.secrets\.versions\.access\) NOT_FOUND:",
        read.stderr, re.MULTILINE,
    )
    if read.returncode and not missing:
        raise RuntimeError(f"{optional.label} secret read failed. Check Secret Manager access and version state.")
    value = read.stdout.strip() if read.returncode == 0 else ""
    return apply_secret(config, optional, value)


def apply_secret(config, optional, value):
    secret = {
        "apiVersion": "v1", "kind": "Secret",
        "metadata": {"name": optional.name, "namespace": config["namespace"]},
        "type": "Opaque",
        # Empty data removes a previously owned key when no version exists.
        "data": {optional.key: base64.b64encode(value.encode()).decode()} if value else {},
    }
    applied = subprocess.run([
        "kubectl", "apply", "--server-side", "--field-manager=ciproxy-deploy", "-f", "-",
        "-o", "jsonpath={.metadata.resourceVersion}",
    ], input=json.dumps(secret), text=True, capture_output=True)
    if applied.returncode:
        raise RuntimeError(f"{optional.label} secret apply failed. Check namespace Secret access.")
    # Only public metadata reaches Helm. The checksum never hashes the payload.
    return applied.stdout.strip()
