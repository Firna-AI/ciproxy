# SPDX-License-Identifier: Apache-2.0
"""Extract and install a real archive through native scratch path rendering."""

import io
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tarfile
import tempfile

from boat_build_tmp import relocate_temp_paths
from _checks_.boat_workdir import finalize_fixture


def check_tmp(case, workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    temporary = root / "opt/ciproxy-build/tmp"
    temporary.mkdir(parents=True)
    provider = root / "provider-tmp"
    provider.mkdir()
    (provider / "keep.txt").write_text("provider data\n")
    runtime = root / "runtime"
    runtime.mkdir()
    archive = temporary / "azcopy.tar.gz"
    with tarfile.open(archive, "w:gz") as output:
        for name, content, mode in [("azcopy", b"#!/bin/sh\nprintf 'archive executable\\n'\n", 0o755),
                                    ("NOTICE.txt", b"fixture notice\n", 0o644)]:
            entry = tarfile.TarInfo("azcopy_linux_amd64_10.34.1/" + name)
            entry.size, entry.mode = len(content), mode
            output.addfile(entry, io.BytesIO(content))
    tools = root / "tools"
    tools.mkdir()
    for name in ("tar", "install"):
        executable = shutil.which(name)
        path = tools / name
        path.write_text(f"#!{sys.executable}\nimport subprocess,sys\narguments=[]\n"
                        "for value in sys.argv[1:]:\n"
                        f"    if value=='/tmp': value={str(provider)!r}\n"
                        f"    elif value.startswith('/tmp/'): value={str(provider)!r}+value[4:]\n"
                        "    arguments.append(value)\n"
                        f"raise SystemExit(subprocess.run([{executable!r},*arguments]).returncode)\n")
        path.chmod(0o755)
    probe = tools / "probe"
    probe.write_text(f"#!{sys.executable}\nimport json,os,sys\n"
                     "with open(os.environ['PROBE_OUTPUT'],'w') as output: json.dump(sys.argv[1:],output)\n")
    probe.chmod(0o755)
    script = r'''tar xzf "$ARCHIVE" --strip-components=1 -C /tmp
install /tmp/azcopy "$RUNTIME/azcopy"
probe /tmp /tmp/a '/tmp' '/tmp/a' "/tmp" "/tmp/a" /tmpfoo https://example.invalid/tmp/a 'https://example.invalid/?next=/tmp/a'
cat > "$RUNTIME/config" <<'CONFIG'
runtime_scratch=/tmp/runtime
CONFIG
set_etc_environment_variable() { printf '%s=%s\n' "$1" "$2" > "$RUNTIME/environment"; }
set_etc_environment_variable RUNTIME_TMP /tmp
printf '%s\n' "/tmp/runtime" > "$RUNTIME/printf-config"
'''
    rendered = relocate_temp_paths(script, str(temporary))
    result = subprocess.run(["bash", "-e", "-c", rendered], cwd=temporary, env={**os.environ,
        "PATH": str(tools) + os.pathsep + os.environ["PATH"], "ARCHIVE": str(archive),
        "RUNTIME": str(runtime), "PROBE_OUTPUT": str(root / "probe.json")}, capture_output=True, text=True)
    case.assertEqual(result.returncode, 0, result.stdout + result.stderr)
    case.assertEqual(subprocess.check_output([str(runtime / "azcopy")], text=True), "archive executable\n")
    case.assertEqual(json.loads((root / "probe.json").read_text()), [
        str(temporary), str(temporary / "a"), str(temporary), str(temporary / "a"),
        str(temporary), str(temporary / "a"), "/tmpfoo", "https://example.invalid/tmp/a",
        "https://example.invalid/?next=/tmp/a"])
    case.assertEqual((runtime / "config").read_text(), "runtime_scratch=/tmp/runtime\n")
    case.assertEqual((runtime / "environment").read_text(), "RUNTIME_TMP=/tmp\n")
    case.assertEqual((runtime / "printf-config").read_text(), "/tmp/runtime\n")
    case.assertEqual((temporary / "NOTICE.txt").read_text(), "fixture notice\n")
    finalize_fixture(case, root)
    case.assertFalse(temporary.parent.exists(), "Finalization must remove extracted build-only files.")
    case.assertEqual((provider / "keep.txt").read_text(), "provider data\n")
    case.assertTrue((runtime / "azcopy").is_file())
