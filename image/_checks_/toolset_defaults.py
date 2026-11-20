# SPDX-License-Identifier: Apache-2.0
"""Keep upstream selector forms while installing the exact pinned Node version."""

import fnmatch
import json
import os
import subprocess

from _checks_.recipe_fixture import rendered


def check_node_default(case, workspace):
    root, upstream, _ = rendered(workspace)
    toolset_path = upstream / "images/ubuntu/toolsets/toolset-2404.json"
    toolset = json.loads(toolset_path.read_text())
    case.assertEqual(toolset["node"]["default"], "22")
    defaults = {tool["name"]: tool.get("default") for tool in toolset["toolcache"]}
    go_default = defaults.pop("go")
    case.assertEqual(defaults, {"Python": None, "node": None, "Ruby": None, "PyPy": None})
    go = next(tool for tool in toolset["toolcache"] if tool["name"] == "go")
    case.assertIn(go_default, go["versions"])
    case.assertTrue(fnmatch.fnmatchcase(go_default, "1.24.*"))
    tools, home = root / "tools", root / "home"
    tools.mkdir()
    home.mkdir()
    helpers = root / "helpers"
    helpers.mkdir()
    (helpers / "install.sh").write_text('get_toolset_value() { jq -r "$1" "$FIXTURE_TOOLSET"; }\n'
                                       'invoke_tests() { :; }\n')
    curl = tools / "curl"
    curl.write_text('''#!/bin/sh
while [ "$#" -gt 0 ]; do
    if [ "$1" = -o ]; then
        shift
        cat > "$1" <<'NODE_INSTALL'
printf '%s\\n' "$1" > "$NODE_VERSION_REQUEST"
NODE_INSTALL
    fi
    shift
done
''')
    curl.chmod(0o755)
    for name in ("npm", "ln", "sudo"):
        tool = tools / name
        tool.write_text("#!/bin/sh\nexit 0\n")
        tool.chmod(0o755)
    requested = root / "node-version"
    script = upstream / "images/ubuntu/scripts/build/install-nodejs.sh"
    result = subprocess.run(["bash", "-e", str(script)], capture_output=True, text=True,
                            env={**os.environ, "PATH": str(tools) + os.pathsep + os.environ["PATH"],
                                 "HOME": str(home), "HELPER_SCRIPTS": str(helpers),
                                 "FIXTURE_TOOLSET": str(toolset_path), "NODE_VERSION_REQUEST": str(requested)})
    case.assertEqual(result.returncode, 0, result.stdout + result.stderr)
    case.assertEqual(requested.read_text(), "22.23.4\n")
