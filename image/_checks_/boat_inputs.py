# SPDX-License-Identifier: Apache-2.0
"""Exercise resume against immutable source and mutable installer output."""

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
    (root / "plan.json").write_text(json.dumps(make_plan(upstream)))
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
    later.write_text(body.replace("NAME", "changed"))
    run(True)
    expected += "changed:shared1:1\n"
    case.assertEqual(trace.read_text(), expected, "A later script must not rerun an earlier installer.")
    helper.write_text('TOKEN=shared2\n')
    run(True)
    expected += "first:shared2:1\nchanged:shared2:1\n"
    case.assertEqual(trace.read_text(), expected, "A real shared helper change must rerun its consumers.")
    toolset.write_text('{"version":"2"}\n')
    run(True)
    expected += "first:shared2:2\nchanged:shared2:2\n"
    case.assertEqual(trace.read_text(), expected, "A real toolset change must rerun its consumers.")
