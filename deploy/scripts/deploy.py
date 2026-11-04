#!/usr/bin/env python3
"""Deploy prepared foundation resources. This command changes live resources."""

import argparse
import json
import os
from pathlib import Path

from deploy_support import (
    helm_values, neg_zones, run, runtime_configuration,
    sync_release_secret, terraform_init,
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("environment", choices=["main-preview", "production"])
    args = parser.parse_args()
    digest, runtime_enabled, control_image = runtime_configuration(os.environ)
    environment = args.environment
    project = "ciproxy" if environment == "production" else "internal-498318"
    root = Path("deploy/environments") / environment
    scratch = Path(".context/deploy") / environment
    scratch.mkdir(parents=True, exist_ok=True)
    bucket = f"ciproxy-{environment}-tfstate-{project}"

    # CI reads the foundation state. Its roles omit SQL, KMS, VPC, and
    # service-account administration.
    terraform_init(root, bucket, f"{environment}/foundation")
    config = json.loads(run("terraform", f"-chdir={root}", "output", "-json",
                            "deployment", capture=True))
    if not runtime_enabled:
        print("Control-plane runtime stays disabled. The current binary has no HTTP service.",
              flush=True)

    run("gcloud", "container", "clusters", "get-credentials", config["cluster_name"],
        f"--region={config['region']}", f"--project={project}", "--dns-endpoint")
    current_runtime = run("kubectl", "get", "deployment", "control-plane",
                          "--namespace=ciproxy", "--ignore-not-found", "-o", "name", capture=True)
    if current_runtime.strip() and not runtime_enabled:
        if os.environ.get("CONTROL_PLANE_RUNTIME_READY") != "false":
            raise ValueError("A running control plane requires true or explicit false readiness.")
    namespace = {"apiVersion": "v1", "kind": "Namespace", "metadata": {"name": "ciproxy"}}
    run("kubectl", "apply", "--server-side", "--field-manager=ciproxy-deploy", "-f", "-",
        input_text=json.dumps(namespace))
    sync_release_secret(config)

    release, ciproxy = helm_values(config, digest, runtime_enabled, control_image)
    ciproxy["controlPlane"]["cdnSigningKeyName"] = os.environ.get("CDN_SIGNING_KEY_NAME", "")
    release_file = scratch / "release-values.json"
    ciproxy_file = scratch / "ciproxy-values.json"
    release_file.write_text(json.dumps(release))
    ciproxy_file.write_text(json.dumps(ciproxy))
    run("helm", "upgrade", "--install", "cli-release-server", "deploy/helm/cli-release-server",
        "--namespace=ciproxy", "--values=deploy/helm/release-values.yaml",
        f"--values={release_file}", "--wait", "--timeout=10m", "--history-max=5")
    run("helm", "upgrade", "--install", "ciproxy", "deploy/helm/ciproxy",
        "--namespace=ciproxy", f"--values={ciproxy_file}",
        "--wait", "--timeout=10m", "--history-max=5")

    for package, directory in [("site", "ts/site/dist"), ("dashboard", "ts/app/dist")]:
        run("python3", "deploy/scripts/publish_static.py", f"--directory={directory}",
            f"--bucket={config['buckets'][package]}")

    neg_zones("release-neg", config["prefix"] + "-release", config["allowed_zones"])
    if runtime_enabled:
        neg_zones("control-plane-neg", config["prefix"] + "-control", config["allowed_zones"])
    edge_variables = scratch / "edge.tfvars.json"
    edge_variables.write_text(json.dumps({
        "control_plane_enabled": runtime_enabled,
        "enable_uptime_checks": os.environ.get("PUBLIC_UPTIME_ENABLED") == "true",
    }))
    edge = root / "edge"
    terraform_init(edge, bucket, f"{environment}/edge")
    plan = (scratch / "edge.tfplan").resolve()
    run("terraform", f"-chdir={edge}", "plan", "-input=false", "-lock-timeout=5m",
        f"-var-file={edge_variables.resolve()}", f"-out={plan}")
    run("terraform", f"-chdir={edge}", "apply", "-input=false", str(plan))
    url_map = run("terraform", f"-chdir={edge}", "output", "-raw", "url_map", capture=True)
    run("gcloud", "compute", "url-maps", "invalidate-cdn-cache", url_map,
        "--path=/*", f"--project={project}")

    summary = (
        f"Deployed {environment} static exports and the release server.\n"
        f"Control-plane runtime enabled: {str(runtime_enabled).lower()}.\n"
        "DNS delegation and the managed certificate may still be pending.\n"
    )
    print(summary)
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as output:
            output.write(summary)


if __name__ == "__main__":
    main()
