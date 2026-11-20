# SPDX-License-Identifier: Apache-2.0
"""Apply native permission setup to a private root with provider entries."""

import importlib.util
import json
import pathlib
import stat
import sys
import tempfile
from unittest.mock import patch


def check_permissions(case, workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    folder = root / "writable"
    folder.mkdir(mode=0o755)
    provider = folder / "provider"
    provider.write_text("provider tool\n")
    provider.chmod(0o755)
    child = folder / "provider-directory"
    child.mkdir(mode=0o755)
    owners = {path: (path.stat().st_uid, path.stat().st_gid) for path in (folder, provider, child)}
    script = pathlib.Path(__file__).resolve().parents[1] / "boat-build-permissions.py"
    spec = importlib.util.spec_from_file_location("permission_fixture", script)
    permissions = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(permissions)
    baseline = root / "permissions.json"
    with patch.object(permissions, "ROOTS", (str(folder),)), patch.object(permissions, "BASELINE", baseline):
        with patch.object(sys, "argv", [str(script), "--capture"]):
            permissions.main()
        case.assertEqual(json.loads(baseline.read_text())[str(folder)], 0o755)
        added = folder / "new-tool"
        added.write_text("new tool\n")
        added.chmod(0o755)
        with patch.object(sys, "argv", [str(script), str(folder)]):
            permissions.main()
    case.assertEqual(stat.S_IMODE(folder.stat().st_mode), 0o777)
    case.assertEqual(stat.S_IMODE(provider.stat().st_mode), 0o755)
    case.assertEqual(stat.S_IMODE(child.stat().st_mode), 0o755)
    case.assertEqual(stat.S_IMODE(added.stat().st_mode), 0o777)
    case.assertEqual({path: (path.stat().st_uid, path.stat().st_gid) for path in owners}, owners)
