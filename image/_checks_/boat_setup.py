# SPDX-License-Identifier: Apache-2.0
"""Run and resume native directory setup on real private files."""

import json
import pathlib
import subprocess
import tempfile
from unittest.mock import patch

import boat_build_plan


def check_setup(case, workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    upstream = root / "runner-images"
    ubuntu = upstream / "images/ubuntu"
    for folder in (ubuntu / "templates", ubuntu / "scripts/docs-gen", ubuntu / "assets/post-gen",
                   ubuntu / "scripts/tests", upstream / "helpers/software-report-base"):
        folder.mkdir(parents=True)
    docs = ubuntu / "scripts/docs-gen/generate.ps1"
    postgen = ubuntu / "assets/post-gen/cleanup.sh"
    tests = ubuntu / "scripts/tests/Tools.Tests.ps1"
    report_base = upstream / "helpers/software-report-base/report.psm1"
    for path in (docs, postgen, tests, report_base):
        path.write_text("version1\n")
    recipe = '''build {
  provisioner "shell" {
    inline = ["mkdir ${var.image_folder}", "chmod 777 ${var.image_folder}"]
  }
  provisioner "file" {
    sources = ["${path.root}/../assets/post-gen", "${path.root}/../scripts/tests", "${path.root}/../scripts/docs-gen"]
    destination = "${var.image_folder}"
  }
  provisioner "file" {
    source = "${path.root}/../../../helpers/software-report-base"
    destination = "${var.image_folder}/docs-gen/"
  }
  provisioner "shell" {
    inline = ["mv ${var.image_folder}/docs-gen ${var.image_folder}/SoftwareReport", "mv ${var.image_folder}/post-gen ${var.image_folder}/post-generation"]
  }
}
'''
    (ubuntu / "templates/build.ubuntu-24_04.pkr.hcl").write_text(recipe)
    image = root / "imagegeneration"
    with patch.dict(boat_build_plan.VARIABLES, {"image_folder": str(image)}):
        plan = boat_build_plan.make_plan(upstream)
    (root / "plan.json").write_text(json.dumps(plan))
    executor = pathlib.Path(__file__).resolve().parents[1] / "boat-build-execute.py"

    def run(resume=False):
        result = subprocess.run(["python3", str(executor), str(root), "--no-inventory", *(["--resume"] if resume else [])],
                                capture_output=True, text=True)
        case.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def interrupted(index):
        report = json.loads((root / "report.json").read_text())
        report["provisioners"][index]["status"] = "running"
        (root / "report.json").write_text(json.dumps(report))

    run()
    case.assertEqual((image / "SoftwareReport/generate.ps1").read_text(), "version1\n")
    interrupted(0)
    run(True)
    (image / "SoftwareReport/keep.json").write_text("retained metadata\n")
    (image / "post-generation/keep.txt").write_text("retained post-generation metadata\n")
    tests.write_text("version2\n")
    run(True)
    case.assertEqual((image / "tests/Tools.Tests.ps1").read_text(), "version2\n")
    case.assertFalse((image / "docs-gen").exists())
    case.assertFalse((image / "post-gen").exists())
    case.assertEqual((image / "SoftwareReport/keep.json").read_text(), "retained metadata\n")
    case.assertEqual((image / "post-generation/keep.txt").read_text(), "retained post-generation metadata\n")
    for path in (docs, postgen, report_base):
        path.write_text("version2\n")
    run(True)
    case.assertEqual((image / "SoftwareReport/generate.ps1").read_text(), "version2\n")
    case.assertEqual((image / "SoftwareReport/software-report-base/report.psm1").read_text(), "version2\n")
    case.assertEqual((image / "post-generation/cleanup.sh").read_text(), "version2\n")
    interrupted(-1)
    run(True)
    case.assertEqual((image / "SoftwareReport/keep.json").read_text(), "retained metadata\n")
    case.assertEqual((image / "post-generation/keep.txt").read_text(), "retained post-generation metadata\n")
    case.assertFalse((image / "docs-gen").exists())
    case.assertFalse((image / "post-gen").exists())
