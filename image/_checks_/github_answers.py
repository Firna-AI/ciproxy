# SPDX-License-Identifier: Apache-2.0
"""Refresh mutable release answers without replaying unchanged recipe steps."""

import contextlib
import hashlib
import importlib.util
import io
import json
import os
import pathlib
import subprocess
import tarfile
import types
import urllib.parse
from unittest.mock import patch

from boat_build_bundle import bundle, checksum, read_pins
from _checks_.github_lookup import responses
from _checks_.recipe_fixture import rendered


def check_answer_refresh(case, workspace):
    root, _, _ = rendered(workspace)
    source = root / "upstream"
    templates = source / "images/ubuntu/templates"
    templates.mkdir()
    (templates / "build.ubuntu-24_04.pkr.hcl").write_text('''build {
  provisioner "file" {
    source = "${path.root}/../scripts/helpers"
    destination = "/opt/ciproxy-build/installed-helpers"
  }
  provisioner "shell" {
    environment_vars = ["HELPER_SCRIPTS=/opt/ciproxy-build/installed-helpers"]
    scripts = ["${path.root}/../scripts/build/install-packer.sh"]
  }
}
''')
    definition = pathlib.Path(__file__).resolve().parents[1]
    pins = read_pins(definition)
    archive = root / "upstream.tar.gz"
    with tarfile.open(archive, "w:gz") as output:
        output.add(source, arcname="runner-images-" + pins["RUNNER_IMAGES_COMMIT"])
    artifact = root / "artifact"
    artifact.write_bytes(b"fixture artifact\n")
    pins.update(RUNNER_IMAGES_SHA256=checksum(archive), RUNNER_SHA256=checksum(artifact))
    overlay = root / "worker-overlay"
    overlay.mkdir()
    (overlay / "stock-deps.sha256").write_text("fixture\n")
    args = types.SimpleNamespace(upstream_archive=archive, runner=artifact, guest=artifact,
                                 cache=artifact, worker=overlay, reference=artifact,
                                 metadata_cache=None, output=root / "bundle.tar.gz")
    _, values = responses(root)
    release = values["repos/Kitware/CMake/releases?per_page=100&page=2"][0]
    release.update(body="Release text", reactions={"total_count": 1})
    release["assets"][0]["download_count"] = 1

    def open_fixture(request, timeout):
        parsed = urllib.parse.urlsplit(request.full_url)
        route = parsed.path.lstrip("/") + ("?" + parsed.query if parsed.query else "")
        value = values.get(route, {"tag_name": route.rsplit("/", 1)[-1], "assets": []})
        return io.BytesIO(json.dumps(value).encode())

    build = root / "build"
    build.mkdir()
    tools = root / "refresh-tools"
    tools.mkdir()
    for name, status in (("systemctl", 1), ("systemd-run", 0)):
        tool = tools / name
        tool.write_text("#!/bin/sh\nexit " + str(status) + "\n")
        tool.chmod(0o755)
    refresher = root / "refresh.sh"
    refresher.write_text((definition / "boat-build-refresh.sh").read_text().replace(
        "/opt/ciproxy-build", str(build)))
    spec = importlib.util.spec_from_file_location("answers_executor", definition / "boat-build-execute.py")
    executor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(executor)

    def refresh(resume):
        with patch("boat_build_bundle.read_pins", return_value=pins), \
                patch("pin_recipe.urllib.request.urlopen", side_effect=open_fixture), \
                contextlib.redirect_stdout(io.StringIO()):
            bundle(args)
        if resume:
            result = subprocess.run(["bash", str(refresher), str(args.output), checksum(args.output)],
                                    env={**os.environ, "PATH": str(tools) + os.pathsep + os.environ["PATH"]},
                                    capture_output=True, text=True)
            case.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            with tarfile.open(args.output) as incoming:
                incoming.extractall(build)
        calls = []
        with patch.dict(os.environ), \
                patch.object(executor, "run_step", side_effect=lambda step, output: calls.append(step["id"]) or 0), \
                contextlib.redirect_stdout(io.StringIO()):
            case.assertEqual(executor.execute(build, resume=resume, inventory=False), 0)
        report = json.loads((build / "report.json").read_text())
        case.assertEqual(report["status"], "passed")
        return [step["input_sha256"] for step in report["provisioners"]], calls

    identities, initial = refresh(False)
    case.assertEqual(len(initial), 5)
    changes = (
        ("download count", lambda: release["assets"][0].update(download_count=2)),
        ("reaction count", lambda: release["reactions"].update(total_count=2)),
        ("new CMake release", lambda: values.update({"repos/Kitware/CMake/releases?per_page=100&page=1": [
            {"tag_name": "v10.0.0", "assets": []},
            *values["repos/Kitware/CMake/releases?per_page=100&page=1"][:-1]]})),
        ("release text", lambda: release.update(body="Updated release text")),
    )
    for name, change in changes:
        change()
        actual, repeated = refresh(True)
        with case.subTest(change=name, check="identities"):
            case.assertEqual(actual, identities, "Release answer changes must preserve every step identity.")
        with case.subTest(change=name, check="refresh"):
            case.assertEqual(repeated, [], "A refresh must reuse every passed step.")
    legacy_cache = build / "runner-images/images/ubuntu/scripts/helpers/github-cache"
    legacy_cache.mkdir(exist_ok=True)
    (legacy_cache / "old.json").write_text("{}")
    args.metadata_cache = root / "provided-answers"
    args.metadata_cache.mkdir()
    route = "repos/Kitware/CMake/releases?per_page=100&page=2"
    key = hashlib.sha256(route.encode()).hexdigest() + ".json"
    (args.metadata_cache / key).write_text(json.dumps([release]))
    actual, repeated = refresh(True)
    case.assertEqual(actual, identities)
    case.assertEqual(repeated, [])
    case.assertEqual(json.loads((build / "github-answers" / key).read_text()), [release])
