# SPDX-License-Identifier: Apache-2.0
"""Exercise chained resume with immutable sources and mutable installer output."""

import json
import pathlib
import subprocess
import tempfile

from boat_build_plan import make_plan


def check_inputs(case, workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    upstream = root / "runner-images"
    ubuntu = upstream / "images/ubuntu"
    for folder in ("templates", "scripts/build", "scripts/helpers", "toolsets"):
        (ubuntu / folder).mkdir(parents=True)
    helpers, installers = root / "helpers", root / "installers"
    helper = ubuntu / "scripts/helpers/shared.sh"
    helper.write_text('TOKEN=shared1\n')
    toolset = ubuntu / "toolsets/toolset-2404.json"
    toolset.write_text('{"version":"1"}\n')
    trace = root / "trace"
    body = '''source "$HELPER_SCRIPTS/shared.sh"
version=$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["version"])' "$INSTALLER_SCRIPT_FOLDER/toolset.json")
printf 'NAME:%s:%s\\n' "$TOKEN" "$version" >> "$TRACE"
'''
    first = ubuntu / "scripts/build/first.sh"
    first.write_text(body.replace("NAME", "first") +
                     'printf "generated\\n" >> "$HELPER_SCRIPTS/apt-sources.txt"\n'
                     'touch "$INSTALLER_SCRIPT_FOLDER/generated-binary"\n')
    later = ubuntu / "scripts/build/later.sh"
    later.write_text(body.replace("NAME", "later"))
    recipe = '''build {
  provisioner "file" {
    source = "${path.root}/../scripts/helpers"
    destination = "HELPERS"
  }
  provisioner "file" {
    source = "${path.root}/../scripts/build"
    destination = "INSTALLERS"
  }
  provisioner "file" {
    source = "${path.root}/../toolsets/toolset-2404.json"
    destination = "INSTALLERS/toolset.json"
  }
  provisioner "shell" {
    environment_vars = ["HELPER_SCRIPTS=HELPERS", "INSTALLER_SCRIPT_FOLDER=INSTALLERS", "TRACE=TRACEFILE"]
    scripts = ["${path.root}/../scripts/build/first.sh", "${path.root}/../scripts/build/later.sh"]
  }
}
'''.replace("=HELPERS", "=" + str(helpers)).replace('"HELPERS"', json.dumps(str(helpers)))
    recipe = recipe.replace("=INSTALLERS", "=" + str(installers)).replace('"INSTALLERS', '"' + str(installers))
    recipe = recipe.replace("TRACEFILE", str(trace))
    (ubuntu / "templates/build.ubuntu-24_04.pkr.hcl").write_text(recipe)
    plan = make_plan(upstream)
    later_input = root / "later-input"
    later_input.write_text("first input\n")
    plan["steps"][-1]["inputs"].append(str(later_input))
    (root / "plan.json").write_text(json.dumps(plan))
    executor = pathlib.Path(__file__).resolve().parents[1] / "boat-build-execute.py"

    def run(resume=False):
        command = ["python3", str(executor), str(root), "--no-inventory"]
        result = subprocess.run(command + (["--resume"] if resume else []), capture_output=True, text=True)
        case.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    run()
    expected = "first:shared1:1\nlater:shared1:1\n"
    case.assertEqual(trace.read_text(), expected)
    run(True)
    case.assertEqual(trace.read_text(), expected, "Mutable installer output must not rerun passed steps.")
    later_input.write_text("changed input\n")
    run(True)
    expected += "later:shared1:1\n"
    case.assertEqual(trace.read_text(), expected, "A later input must not rerun an earlier installer.")
    later.write_text(body.replace("NAME", "changed"))
    run(True)
    expected += "first:shared1:1\nchanged:shared1:1\n"
    case.assertEqual(trace.read_text(), expected, "Changed copied source must replay every step after its copy.")
    helper.write_text('TOKEN=shared2\n')
    run(True)
    expected += "first:shared2:1\nchanged:shared2:1\n"
    case.assertEqual(trace.read_text(), expected, "A real shared helper change must rerun its consumers.")
    toolset.write_text('{"version":"2"}\n')
    run(True)
    expected += "first:shared2:2\nchanged:shared2:2\n"
    case.assertEqual(trace.read_text(), expected, "A real toolset change must rerun its consumers.")


def check_bootstrap_chain(case, workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    trace = root / "trace"
    bootstrap_input, skipped_input = root / "bootstrap-input", root / "skipped-input"
    bootstrap_input.write_text("first bootstrap input\n")
    skipped_input.write_text("first skipped input\n")
    environment = {"TRACE": str(trace)}
    common = {"type": "shell", "provisioner": 0, "environment": environment}
    steps = [{**common, "id": "000-bootstrap", "inputs": [str(bootstrap_input)],
              "inline": ['printf "bootstrap\\n" >> "$TRACE"']},
             {**common, "id": "001-skipped", "inputs": [str(skipped_input)],
              "inline": ["exit 99"], "skip": "Provider-only fixture step."},
             {**common, "id": "002-tool", "inline": ['printf "tool\\n" >> "$TRACE"']},
             {**common, "id": "003-last", "inline": ['printf "last\\n" >> "$TRACE"']}]
    plan = root / "plan.json"
    plan.write_text(json.dumps({"steps": steps}))
    executor = pathlib.Path(__file__).resolve().parents[1] / "boat-build-execute.py"

    def run(resume=False):
        result = subprocess.run(["python3", str(executor), str(root), "--no-inventory",
                                 *(["--resume"] if resume else [])], capture_output=True, text=True)
        case.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    run()
    expected = "bootstrap\ntool\nlast\n"
    case.assertEqual(trace.read_text(), expected)
    case.assertEqual(run(True).stdout, "", "An unchanged resume must reuse every passed step.")
    case.assertEqual(trace.read_text(), expected)
    bootstrap_input.write_text("changed bootstrap input\n")
    run(True)
    expected += "bootstrap\ntool\nlast\n"
    case.assertEqual(trace.read_text(), expected, "A bootstrap input change must replay every later passed step.")
    skipped_input.write_text("changed skipped input\n")
    run(True)
    expected += "tool\nlast\n"
    case.assertEqual(trace.read_text(), expected, "A skipped step's changed input must join the identity chain.")
    steps.insert(1, {**common, "id": "new-skipped", "inline": ["exit 99"], "skip": "New fixture step."})
    plan.write_text(json.dumps({"steps": steps}))
    run(True)
    expected += "tool\nlast\n"
    case.assertEqual(trace.read_text(), expected, "A new earlier step must replay later passed steps.")


def check_failed_step(case, workspace):
    executor = pathlib.Path(__file__).resolve().parents[1] / "boat-build-execute.py"
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    upstream = root / "runner-images"
    templates = upstream / "images/ubuntu/templates"
    scripts = upstream / "images/ubuntu/scripts/build"
    helpers = upstream / "images/ubuntu/scripts/helpers"
    for folder in (templates, scripts, helpers):
        folder.mkdir(parents=True, exist_ok=True)
    trace, flag = root / "trace", root / "blocked"
    flag.touch()
    (helpers / "helper.txt").write_text("helper data\n")
    (scripts / "toolset.json").write_text('{"fixture": true}\n')
    (scripts / "first.sh").write_text('printf "first\\n" >> "$TRACE"\n')
    (scripts / "second.sh").write_text('printf "second\\n" >> "$TRACE"\n'
                                       'test ! -e "$BLOCKED" || exit 17\n')
    recipe = '''build {
  provisioner "file" {
source = "${path.root}/../scripts/helpers"
destination = "COPIED"
  }
  provisioner "file" {
source = "${path.root}/../scripts/build/toolset.json"
destination = "TOOLSET"
  }
  provisioner "shell" {
environment_vars = ["TRACE=TRACEFILE", "BLOCKED=FLAGFILE"]
scripts = ["${path.root}/../scripts/build/first.sh", "${path.root}/../scripts/build/second.sh"]
  }
  provisioner "shell" {
inline = ["printf 'last\\n' >> TRACEFILE"]
  }
}
'''.replace("COPIED", str(root / "copied")).replace("TOOLSET", str(root / "copied.json"))
    recipe = recipe.replace("TRACEFILE", str(trace)).replace("FLAGFILE", str(flag))
    (templates / "build.ubuntu-24_04.pkr.hcl").write_text(recipe)
    plan = make_plan(upstream)
    artifact = root / "artifact"
    artifact.write_text("first input\n")
    plan["steps"][-1]["inputs"] = [str(artifact)]
    (root / "plan.json").write_text(json.dumps(plan))

    def run(resume=False):
        command = ["python3", str(executor), str(root), "--no-inventory"]
        observed = subprocess.run(command + (["--resume"] if resume else []),
                                  capture_output=True, text=True)
        return observed, json.loads((root / "report.json").read_text())

    initial, report = run()
    case.assertEqual(initial.returncode, 1)
    case.assertEqual([step["status"] for step in report["provisioners"]],
                     ["passed", "passed", "passed", "failed", "pending"])
    case.assertEqual(report["provisioners"][3]["exit_code"], 17)
    case.assertEqual((root / "copied/helper.txt").read_text(), "helper data\n")
    case.assertEqual(json.loads((root / "copied.json").read_text()), {"fixture": True})
    case.assertEqual(trace.read_text(), "first\nsecond\n")
    flag.unlink()
    resumed, report = run(True)
    case.assertEqual(resumed.returncode, 0, resumed.stderr)
    case.assertEqual(report["status"], "passed")
    case.assertEqual(trace.read_text(), "first\nsecond\nsecond\nlast\n")
    (scripts / "first.sh").write_text('printf "changed\\n" >> "$TRACE"\n')
    changed, report = run(True)
    case.assertEqual(changed.returncode, 0, changed.stderr)
    case.assertEqual(trace.read_text(), "first\nsecond\nsecond\nlast\nchanged\nsecond\nlast\n")
    artifact.write_text("new input\n")
    refreshed, report = run(True)
    case.assertEqual(refreshed.returncode, 0, refreshed.stderr)
    case.assertEqual(trace.read_text(), "first\nsecond\nsecond\nlast\nchanged\nsecond\nlast\nlast\n")
