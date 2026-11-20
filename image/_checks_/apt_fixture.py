# SPDX-License-Identifier: Apache-2.0
"""Use real APT with a private package index, status file, and preferences."""

import pathlib
import shutil
import subprocess
import tempfile
import types


def repository(case, workspace, packages, preferences, installed=""):
    apt_get, apt_cache = shutil.which("apt-get"), shutil.which("apt-cache")
    if not apt_get or not apt_cache:
        case.skipTest("APT is required for the package solver regression.")
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    for name in ("lists/partial", "cache/archives/partial", "repo", "etc/parts"):
        (root / name).mkdir(parents=True)
    (root / "repo/Packages").write_text(packages)
    (root / "status").write_text(installed)
    (root / "etc/sources.list").write_text(f"deb [trusted=yes] file:{root}/repo ./\n")
    (root / "etc/preferences").write_text(preferences)
    prefix = pathlib.Path(apt_get).resolve().parents[1]
    options = {
        "APT::Architecture": "amd64", "APT::System": "Debian dpkg interface",
        "APT::Sandbox::User": "", "Acquire::Languages": "none",
        "Dir::Etc::main": str(root / "etc/apt.conf"), "Dir::Etc::parts": str(root / "etc/parts"),
        "Dir::Etc::sourcelist": str(root / "etc/sources.list"), "Dir::Etc::sourceparts": str(root / "etc/parts"),
        "Dir::Etc::preferences": str(root / "etc/preferences"), "Dir::Etc::preferencesparts": str(root / "etc/parts"),
        "Dir::State::status": str(root / "status"), "Dir::State::lists": str(root / "lists"),
        "Dir::Cache": str(root / "cache"), "Dir::Cache::pkgcache": "", "Dir::Cache::srcpkgcache": "",
        "Dir::Log": str(root), "Dir::Bin::methods": str(prefix / "lib/apt/methods"),
        "Dir::Bin::dpkg": str(prefix / "bin/dpkg"), "Dir::dpkg::cputable": str(prefix / "share/dpkg/cputable"),
        "Dir::dpkg::tupletable": str(prefix / "share/dpkg/tupletable"),
    }
    configuration = [arg for key, value in options.items() for arg in ("-o", key + "=" + value)]

    def run(binary, *arguments):
        return subprocess.run([binary, *configuration, *arguments], capture_output=True, text=True)

    return types.SimpleNamespace(root=root, apt_get=apt_get, apt_cache=apt_cache, configuration=configuration, run=run)
