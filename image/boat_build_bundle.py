# SPDX-License-Identifier: Apache-2.0
"""Make a verified, credential-free native build input bundle."""

import hashlib
import json
import pathlib
import shutil
import tarfile
import tempfile

from boat_build_native import adapt
from boat_build_plan import make_plan
from pin_recipe import render, warm_github_cache


def read_pins(definition):
    return dict(line.split("=", 1) for line in (definition / "pins.env").read_text().splitlines()
                if line and not line.startswith("#"))


def checksum(path):
    result = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            result.update(chunk)
    return result.hexdigest()


def bundle(args):
    definition = pathlib.Path(__file__).parent.resolve()
    pins = read_pins(definition)
    for path, expected in [(args.upstream_archive, pins["RUNNER_IMAGES_SHA256"]),
                           (args.runner, pins["RUNNER_SHA256"])]:
        if checksum(path) != expected:
            raise RuntimeError(f"Input SHA-256 differs from image/pins.env: {path.name}")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="native-bundle-", dir=args.output.parent) as temporary:
        workspace = pathlib.Path(temporary)
        with tarfile.open(args.upstream_archive, "r:gz") as archive:
            archive.extractall(workspace)
        upstream = workspace / ("runner-images-" + pins["RUNNER_IMAGES_COMMIT"])
        root = workspace / "bundle"
        root.mkdir()
        locked = render(upstream, root / "runner-images")
        answers = root / "github-answers"
        if args.metadata_cache:
            shutil.copytree(args.metadata_cache, answers, dirs_exist_ok=True)
        warm_github_cache(root / "runner-images", locked, answers)
        adaptations = adapt(root / "runner-images")
        shutil.copytree(definition, root / "definition",
                        ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))
        artifacts = root / "artifacts"
        artifacts.mkdir()
        for source, name in [(args.guest, "ciproxy-guest"), (args.cache, "ciproxy-cache-sidecar"),
                             (args.worker, "ciproxy-worker.dll"), (args.runner, "ciproxy-runner.tar.gz")]:
            shutil.copy2(source, artifacts / name)
        shutil.copy2(args.reference, root / "reference-software.json")
        plan = make_plan(root / "runner-images")
        plan.update(image_id=pins["IMAGE_ID"], upstream_commit=pins["RUNNER_IMAGES_COMMIT"],
                    software_pins_sha256=checksum(definition / "software-pins.json"), adaptations=adaptations)
        plan["steps"].insert(0, {"id": "000-native-bootstrap", "provisioner": 0, "type": "shell",
                                  "script": "/opt/ciproxy-build/definition/boat-build-bootstrap.sh",
                                  "inputs": ["/opt/ciproxy-build/definition/boat-build-permissions.py",
                                             "/opt/ciproxy-build/definition/boat-build-sources.py",
                                             "/opt/ciproxy-build/definition/boat-build-developer.py",
                                             "/opt/ciproxy-build/definition/boat-developer-tools.json"],
                                  "interpreter": "bash", "user": "root", "environment": {}})
        plan["steps"].append({"id": "085-ciproxy-artifacts", "provisioner": 99, "type": "shell",
                              "inputs": ["/opt/ciproxy-build/definition", "/opt/ciproxy-build/artifacts",
                                  "/opt/ciproxy-build/runner-images/images/ubuntu/assets/post-gen/cleanup-logs.sh"],
                              "user": "root", "environment": {}, "inline": [
                                  "mkdir -p /tmp/ciproxy-image",
                                  "cp -a /opt/ciproxy-build/definition/. /tmp/ciproxy-image/",
                                  "cp /opt/ciproxy-build/artifacts/* /tmp/",
                                  "install -m 0755 /opt/ciproxy-build/runner-images/images/ubuntu/assets/post-gen/cleanup-logs.sh /opt/post-generation/cleanup-logs.sh"]})
        plan["steps"].append({"id": "086-boat-provision", "provisioner": 100, "type": "shell",
                              "inputs": ["/opt/ciproxy-build/artifacts", *[
                                  "/opt/ciproxy-build/definition/" + name for name in (
                                      "provision.sh", "boat-build-clean-image.sh", "boat-build-locale.sh",
                                      "boat-build-nvm.py", "boat-runner-entry.sh", "boat-runner-errors.sh",
                                      "boat-runner-stderr.py",
                                      "pins.env", "boat-run.sh", "boat-start.sh", "measure.py",
                                      "metadata-smoke.py", "runner-entry.sh", "docker-attach.sh",
                                      "runner-patch/container-proxy-url.sh", "ciproxy-boat-start.service",
                                      "ciproxy-boat-guest.service")]],
                              "script": "/opt/ciproxy-build/definition/boat-provision.sh", "interpreter": "bash",
                              "user": "root", "environment": {}})
        for step in plan["steps"]:
            if step["type"] == "shell":
                step["working_directory"] = "/opt/ciproxy-build/tmp"
        (root / "plan.json").write_text(json.dumps(plan, indent=2) + "\n")
        with tarfile.open(args.output, "w:gz", compresslevel=1) as archive:
            for path in sorted(root.iterdir()):
                archive.add(path, arcname=path.name)
    print(json.dumps({"bundle": str(args.output), "sha256": checksum(args.output),
                      "steps": len(plan["steps"]), "image_id": pins["IMAGE_ID"]}))
