# SPDX-License-Identifier: Apache-2.0
"""Retry the real rendered PyPy installer after a controlled pip download failure."""

import io
import json
import pathlib
import subprocess
import tarfile

from boat_build_native import adapt
from _checks_.recipe_fixture import rendered


def check_pypy_resume(case, workspace):
    root, upstream, _ = rendered(workspace)
    adapt(upstream)
    build = root / "build"
    build.mkdir()
    installer = upstream / "images/ubuntu/scripts/build/install-pypy.sh"
    installer.write_text(installer.read_text().replace("/opt/ciproxy-build", str(build)))
    archive = root / "pypy.tar.gz"
    package = "pypy3.9-v7.3.16-linux64"
    interpreter = b'''#!/bin/bash
if [[ $1 == -c ]]; then
    if [[ $2 == *pypy_version_info* ]]; then printf '7.3.16\\n'; else printf '3.9.19\\n'; fi
elif [[ $2 == pip ]]; then
    curl https://example.invalid/pip-download || exit $?
    touch "$PIP_SUCCESS"
fi
'''
    with tarfile.open(archive, "w:gz") as output:
        entry = tarfile.TarInfo(package + "/bin/pypy3")
        entry.size, entry.mode = len(interpreter), 0o755
        output.addfile(entry, io.BytesIO(interpreter))
    tools, helpers = root / "tools", root / "helpers"
    tools.mkdir()
    helpers.mkdir()
    (helpers / "install.sh").write_text('''download_with_retry() {
    if [[ -f "$BLOCK_ARCHIVE" ]]; then return 18; fi
    printf '%s\\n' "$ARCHIVE"
}
get_toolset_value() { printf '3.9.19\\n'; }
''')
    for name, body in (("chown", "exit 0"), ("curl", 'if [[ -f "$BLOCK_PIP" ]]; then printf "Controlled pip download failure\\n" >&2; exit 17; fi')):
        path = tools / name
        path.write_text("#!/bin/bash\n" + body + "\n")
        path.chmod(0o755)
    blocked, archive_blocked, pip_success = root / "blocked-pip", root / "blocked-archive", root / "pip-success"
    blocked.touch()
    refresh = root / "refresh-input"
    refresh.write_text("first\n")
    environment = {"FIXTURE_TOOLS": str(tools), "INSTALLER": str(installer), "HELPER_SCRIPTS": str(helpers),
                   "AGENT_TOOLSDIRECTORY": str(root / "toolcache"), "ARCHIVE": str(archive),
                   "BLOCK_PIP": str(blocked), "BLOCK_ARCHIVE": str(archive_blocked), "PIP_SUCCESS": str(pip_success)}
    step = {"id": "pypy", "provisioner": 1, "type": "shell", "environment": environment,
            "inputs": [str(installer), str(refresh)],
            "inline": ['export PATH="$FIXTURE_TOOLS:$PATH"', 'bash -e "$INSTALLER"']}
    (build / "plan.json").write_text(json.dumps({"steps": [step]}))
    executor = pathlib.Path(__file__).resolve().parents[1] / "boat-build-execute.py"

    def run(resume=False):
        return subprocess.run(["python3", str(executor), str(build), "--no-inventory",
                               *(["--resume"] if resume else [])], capture_output=True, text=True)

    first = run()
    case.assertEqual(first.returncode, 1)
    report = json.loads((build / "report.json").read_text())
    case.assertEqual(report["provisioners"][0]["exit_code"], 17)
    version = root / "toolcache/PyPy/3.9.19"
    case.assertTrue((version / "x64/bin/pypy3").is_file())
    case.assertFalse((version / "x64.complete").exists())
    (version / "partial-only").write_text("incomplete state\n")
    blocked.unlink()
    resumed = run(True)
    case.assertEqual(resumed.returncode, 0, (build / "logs/pypy.log").read_text())
    case.assertTrue((version / "x64.complete").is_file())
    case.assertTrue(pip_success.is_file())
    case.assertFalse((version / "partial-only").exists())
    case.assertFalse((version / "x64/x64").exists())
    kept = version / "keep-complete"
    kept.write_text("completed state\n")
    blocked.touch()
    archive_blocked.touch()
    refresh.write_text("force replay\n")
    replayed = run(True)
    case.assertEqual(replayed.returncode, 0, (build / "logs/pypy.log").read_text())
    case.assertEqual(kept.read_text(), "completed state\n")
