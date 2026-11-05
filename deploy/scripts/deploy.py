#!/usr/bin/env python3
"""Deploy workloads or invalidate CDN after a saved edge plan is applied."""

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
    parser.add_argument("--phase", choices=["workloads", "invalidate"], default="workloads")
    args = parser.parse_args()
    environment = args.environment
    project = "ciproxy" if environment == "production" else "internal-498318"
    root = Path("deploy/environments") / environment
    scratch = Path(".context/deploy") / environment
    scratch.mkdir(parents=True, exist_ok=True)
    bucket = f"ciproxy-{environment}-tfstate-{project}"

    if args.phase == "invalidate":
        edge = root / "edge"
        terraform_init(edge, bucket, f"{environment}/edge")
        url_map = run("terraform", f"-chdir={edge}", "output", "-raw", "url_map", capture=True).strip()
        run("gcloud", "compute", "url-maps", "invalidate-cdn-cache", url_map,
            "--path=/*", f"--project={project}")
        return

    # Validate runtime inputs only after CI created the first foundation.
    digest, runtime_enabled, control_image = runtime_configuration(os.environ)
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
    summary = (
        f"Deployed {environment} static exports and the release server.\n"
        f"Control-plane runtime enabled: {str(runtime_enabled).lower()}.\n"
        "All configured NEGs are ready. CI can now plan and apply the edge.\n"
    )
    print(summary)
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as output:
            output.write(summary)


if __name__ == "__main__":
    main()
