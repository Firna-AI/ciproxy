"""Cloud commands shared by the deployment entry point."""

import base64
import json
import re
import subprocess
import time


def run(*arguments, capture=False, input_text=None):
    result = subprocess.run(
        arguments, check=True, text=True, input=input_text,
        stdout=subprocess.PIPE if capture else None,
    )
    return result.stdout if capture else None


def terraform_init(target):
    run("terraform", f"-chdir={target.directory}", "init", "-input=false", "-reconfigure",
        *target.arguments())


def runtime_configuration(environment):
    digest = environment.get("RELEASE_SERVER_IMAGE_DIGEST", "")
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", digest):
        raise ValueError("Set RELEASE_SERVER_IMAGE_DIGEST to the prepared image digest.")
    ready = environment.get("CONTROL_PLANE_RUNTIME_READY", "")
    if ready not in ["", "false", "true"]:
        raise ValueError("CONTROL_PLANE_RUNTIME_READY must be true, false, or unset.")
    image = environment.get("CONTROL_PLANE_IMAGE", "")
    if ready == "true" and not re.fullmatch(r"[^@\s]+@sha256:[0-9a-f]{64}", image):
        raise ValueError("A ready runtime requires CONTROL_PLANE_IMAGE pinned by SHA-256.")
    return digest, ready == "true", image


def sync_release_secret(config):
    # The secret never enters a shell argument, file, Terraform state, Helm
    # values, or a last-applied annotation. kubectl receives it over stdin.
    token = run("gcloud", "secrets", "versions", "access", "latest",
                f"--secret={config['release_secret']}",
                f"--project={config['project_id']}", capture=True).strip()
    secret = {
        "apiVersion": "v1", "kind": "Secret",
        "metadata": {"name": "release-github-token", "namespace": config["namespace"]},
        "type": "Opaque",
        "data": {"CLI_RELEASE_GITHUB_TOKEN": base64.b64encode(token.encode()).decode()},
    }
    run("kubectl", "apply", "--server-side", "--field-manager=ciproxy-deploy", "-f", "-",
        input_text=json.dumps(secret))


def neg_zones(service, expected_name, expected_zones, namespace="ciproxy-main-preview"):
    # The controller may take a few minutes to create the NEGs after Helm.
    # Wait for every pre-provisioned zone. A partial status must not allow
    # edge deployment to omit a zone where Autopilot can later place pods.
    expected = set(expected_zones)
    deadline = time.monotonic() + 600
    while time.monotonic() < deadline:
        result = json.loads(run("kubectl", "get", "service", service,
                                f"--namespace={namespace}", "-o", "json", capture=True))
        annotation = result.get("metadata", {}).get("annotations", {}).get(
            "cloud.google.com/neg-status"
        )
        if annotation:
            status = json.loads(annotation)
            if status.get("network_endpoint_groups", {}).get("8080") == expected_name:
                zones = status.get("zones", [])
                if expected.issubset(zones):
                    return sorted(expected)
        time.sleep(5)
    raise RuntimeError(f"GKE did not publish all configured NEG zones for Service {service}.")


def helm_values(config, image_digest, runtime_enabled, control_image):
    release = {
        "affinity": {
            "nodeAffinity": {
                "requiredDuringSchedulingIgnoredDuringExecution": {
                    "nodeSelectorTerms": [{"matchExpressions": [{
                        "key": "topology.kubernetes.io/zone",
                        "operator": "In",
                        "values": config["allowed_zones"],
                    }]}],
                },
            },
        },
        "image": {
            "repository": config["runtime_repository"] + "/cli-release-server",
            "tag": "upstream-30513bd1a6e1@" + image_digest,
        },
        "config": {
            "publicReleaseServerUrl": "https://" + config["hosts"]["releases"],
            "installHosts": [config["hosts"]["releases"]],
        },
    }
    ciproxy = {
        "prefix": config["prefix"],
        "releaseServer": {"instance": config["prefix"] + "-release"},
        "allowedZones": config["allowed_zones"],
        "controlPlane": {
            "enabled": runtime_enabled,
            "image": control_image,
            "serviceAccount": config["control_plane_account"],
            "sqlConnection": config["sql_connection"],
            "sqlUser": config["sql_user"],
            "kmsKey": config["kms_key"],
            "publicUrl": "https://" + config["hosts"]["dashboard"],
            "imageUrlPrefix": "https://" + config["hosts"]["dashboard"] + "/images/",
            "secrets": config["secrets"],
        },
    }
    return release, ciproxy
