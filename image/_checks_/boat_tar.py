# SPDX-License-Identifier: Apache-2.0
"""Extract a real tar archive through the native executor's environment."""

import io
import json
import os
import pathlib
import stat
import subprocess
import tarfile

from boat_build_native import adapt
from _checks_.recipe_fixture import rendered


def check_tar(case, workspace):
    root, upstream, _ = rendered(workspace)
    adaptations = adapt(upstream)
    destination = root / "destination"
    existing = destination / "existing"
    existing.mkdir(parents=True)
    existing.chmod(0o777)
    owner = (existing.stat().st_uid, existing.stat().st_gid)
    archive_path = root / "archive.tar"
    with tarfile.open(archive_path, "w") as archive:
        for name in ("existing", "existing/new-directory"):
            entry = tarfile.TarInfo(name)
            entry.type, entry.mode, entry.uid, entry.gid = tarfile.DIRTYPE, 0o700, 1234, 2345
            archive.addfile(entry)
        payload = b"archive payload\n"
        entry = tarfile.TarInfo("existing/new-directory/file")
        entry.mode, entry.uid, entry.gid, entry.size = 0o644, 1234, 2345, len(payload)
        archive.addfile(entry, io.BytesIO(payload))
    build = root / "build"
    build.mkdir()
    home = root / "home"
    home.mkdir()
    (home / ".profile").write_text("export TAR_OPTIONS=--overwrite-dir\n")
    step = {"id": "extract", "type": "shell", "provisioner": 1,
            "environment": {"ARCHIVE": str(archive_path), "DESTINATION": str(destination), "HOME": str(home),
                            "TAR_OPTIONS": "--overwrite-dir"},
            "inline": ['umask 022', 'tar xf "$ARCHIVE" -C "$DESTINATION"']}
    (build / "plan.json").write_text(json.dumps({"steps": [step], "adaptations": adaptations}))
    executor = pathlib.Path(__file__).resolve().parents[1] / "boat-build-execute.py"
    result = subprocess.run(["python3", str(executor), str(build), "--no-inventory"],
                            capture_output=True, text=True)
    case.assertEqual(result.returncode, 0, result.stdout + result.stderr)
    case.assertEqual(stat.S_IMODE(existing.stat().st_mode), 0o777)
    case.assertEqual((existing.stat().st_uid, existing.stat().st_gid), owner)
    created = existing / "new-directory"
    case.assertEqual(stat.S_IMODE(created.stat().st_mode), 0o700)
    expected_owner = (1234, 2345) if os.geteuid() == 0 else owner
    case.assertEqual((created.stat().st_uid, created.stat().st_gid), expected_owner)
    case.assertEqual((created / "file").read_bytes(), payload)
    report = json.loads((build / "report.json").read_text())
    records = [change for change in report["adaptations"] if "environment" in change]
    case.assertEqual([change["environment"] for change in records], [{"TAR_OPTIONS": "--no-overwrite-dir"}])
    case.assertTrue(records[0]["reason"])
