#!/usr/bin/env python3
"""Deploy workloads or invalidate CDN after a saved edge plan is applied."""

import argparse
import json
import os
from pathlib import Path

from boat_secret import sync_boat_secret
from deploy_support import (
    helm_values, neg_zones, run, runtime_configuration,
    sync_release_secret, terraform_init,
)
from terraform_roots import ENVIRONMENTS, backend
from workload_snapshot import save as save_workload_inputs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("environment", choices=["main-preview", "production"])
    parser.add_argument("--phase", choices=["workloads", "invalidate"], default="workloads")
    parser.add_argument("--target", choices=ENVIRONMENTS, help="Central production preview invalidation only.")
    args = parser.parse_args()
    environment = args.target or args.environment
    if args.target and (args.environment != "production" or args.phase != "invalidate"):
        raise ValueError("Only production can invalidate the promoted preview edge.")
    project = ENVIRONMENTS[environment]
    foundation = backend(environment, "data")
    root = foundation.directory
    scratch = Path(".context/deploy") / environment
    scratch.mkdir(parents=True, exist_ok=True)

    if args.phase == "invalidate":
        edge = backend(environment, "edge")
        terraform_init(edge)
        url_map = run("terraform", f"-chdir={edge.directory}", "output", "-raw", "url_map", capture=True).strip()
        run("gcloud", "compute", "url-maps", "invalidate-cdn-cache", url_map,
            "--path=/*", f"--project={project}")
        return

    # Validate runtime inputs only after CI created the first foundation.
    digest, runtime_enabled, control_image = runtime_configuration(os.environ)
    terraform_init(foundation)
    config = json.loads(run("terraform", f"-chdir={root}", "output", "-json",
                            "deployment", capture=True))
    if not runtime_enabled:
        print("Control-plane runtime stays disabled. Static exports and the release server can deploy.",
              flush=True)

    run("gcloud", "container", "clusters", "get-credentials", config["cluster_name"],
        f"--region={config['region']}", f"--project={project}", "--dns-endpoint")
    current_runtime = run("kubectl", "get", "deployment", "control-plane",
                          f"--namespace={config['namespace']}", "--ignore-not-found", "-o", "name", capture=True)
    if current_runtime.strip() and not runtime_enabled:
        if os.environ.get("CONTROL_PLANE_RUNTIME_READY") != "false":
            raise ValueError("A running control plane requires true or explicit false readiness.")
    sync_release_secret(config)
    boat_revision = sync_boat_secret(config)

    release, ciproxy = helm_values(config, digest, runtime_enabled, control_image, environment)
    ciproxy["controlPlane"]["boatSharedSnapshotRevision"] = boat_revision
    ciproxy["controlPlane"]["cdnSigningKeyName"] = os.environ.get("CDN_SIGNING_KEY_NAME", "")
    ciproxy["controlPlane"]["env"] = [
        {"name": name, "value": os.environ.get(source, "")}
        for name, source in (
            ("GITHUB_APP_ID", "CIPROXY_APP_ID"),
            ("GITHUB_OAUTH_CLIENT_ID", "CIPROXY_APP_CLIENT_ID"),
            ("GITHUB_APP_SLUG", "CIPROXY_APP_SLUG"),
            ("JOB_IMAGE_MANIFEST_PATH", "JOB_IMAGE_MANIFEST_PATH"),
        )
    ] + [
        {"name": "DOWNLOAD_URL", "value": "https://" + config["hosts"]["releases"]},
        {"name": "BUILD_ID", "value": os.environ.get("CIPROXY_SOURCE_SHA", "")},
    ]
    release_file = scratch / "release-values.json"
    ciproxy_file = scratch / "ciproxy-values.json"
    release_file.write_text(json.dumps(release))
    ciproxy_file.write_text(json.dumps(ciproxy))
    run("helm", "upgrade", "--install", config["prefix"] + "-release", "deploy/helm/cli-release-server",
        f"--namespace={config['namespace']}", "--values=deploy/helm/release-values.yaml",
        f"--values={release_file}", "--wait", "--timeout=10m", "--history-max=5")
    run("helm", "upgrade", "--install", config["prefix"], "deploy/helm/ciproxy",
        f"--namespace={config['namespace']}", f"--values={ciproxy_file}",
        "--wait", "--timeout=10m", "--history-max=5")

    for package, directory in [("site", "ts/site/dist"), ("dashboard", "ts/app/dist")]:
        run("python3", "deploy/scripts/publish_static.py", f"--directory={directory}",
            f"--bucket={config['buckets'][package]}")

    neg_zones("release-neg", config["prefix"] + "-release", config["allowed_zones"], config["namespace"])
    if runtime_enabled:
        neg_zones("control-plane-neg", config["prefix"] + "-control", config["allowed_zones"], config["namespace"])
    save_workload_inputs(environment, config, runtime_enabled, control_image)
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
