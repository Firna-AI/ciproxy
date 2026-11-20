# SPDX-License-Identifier: Apache-2.0
"""Resolve a shared family pin with real APT and an isolated file repository."""

import json
import pathlib
import re
import shutil
import subprocess
import tempfile

from pin_packages import apt_preferences


REFERENCE = "255.4-1ubuntu8.18"
NEWER = "255.4-1ubuntu8.19"
FAMILY = ("systemd", "libsystemd-shared", "libsystemd0", "libudev1", "udev", "libnss-systemd",
          "libpam-systemd", "systemd-dev", "systemd-resolved", "systemd-sysv", "systemd-timesyncd",
          "systemd-coredump")


def check_family(case, workspace):
    apt_get, apt_cache = shutil.which("apt-get"), shutil.which("apt-cache")
    if not apt_get or not apt_cache:
        case.skipTest("APT is required for the package-family solver regression.")
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    for name in ("lists/partial", "cache/archives/partial", "repo", "etc/parts"):
        (root / name).mkdir(parents=True)
    packages, installed = [], []
    for name in (*FAMILY, "systemd-hwe-hwdb"):
        versions = (REFERENCE, NEWER) if name in FAMILY else ("255.1.7",)
        for version in versions:
            source = "systemd" if name in FAMILY else "systemd-hwe"
            record = f"Package: {name}\nVersion: {version}\nArchitecture: amd64\nSource: {source}\n"
            if name in ("systemd", "systemd-coredump"):
                record += f"Depends: libsystemd-shared (= {version})\n"
            if name == "libnss-systemd":
                record += f"Depends: systemd (= {version})\n"
            record += "Description: Package family fixture\n"
            packages.append(record + f"Filename: {name}_{version}.deb\nSize: 1\n")
            if version != REFERENCE and name != "systemd-coredump":
                installed.append(record + "Status: install ok installed\n")
    (root / "repo/Packages").write_text("\n".join(packages))
    (root / "status").write_text("\n".join(installed))
    (root / "etc/sources.list").write_text(f"deb [trusted=yes] file:{root}/repo ./\n")
    definition = pathlib.Path(__file__).resolve().parents[1]
    pins = json.loads((definition / "software-apt-pins.json").read_text())["packages"]
    (root / "etc/preferences").write_text(apt_preferences(pins))
    prefix = pathlib.Path(apt_get).resolve().parents[1]
    options = {
        "APT::Architecture": "amd64", "APT::System": "Debian dpkg interface",
        "APT::Sandbox::User": "", "Acquire::Languages": "none",
        "Dir::Etc::main": str(root / "etc/apt.conf"), "Dir::Etc::parts": str(root / "etc/parts"),
        "Dir::Etc::sourcelist": str(root / "etc/sources.list"),
        "Dir::Etc::sourceparts": str(root / "etc/parts"),
        "Dir::Etc::preferences": str(root / "etc/preferences"),
        "Dir::Etc::preferencesparts": str(root / "etc/parts"),
        "Dir::State::status": str(root / "status"), "Dir::State::lists": str(root / "lists"),
        "Dir::Cache": str(root / "cache"), "Dir::Cache::pkgcache": "", "Dir::Cache::srcpkgcache": "",
        "Dir::Log": str(root), "Dir::Bin::methods": str(prefix / "lib/apt/methods"),
        "Dir::Bin::dpkg": str(prefix / "bin/dpkg"),
        "Dir::dpkg::cputable": str(prefix / "share/dpkg/cputable"),
        "Dir::dpkg::tupletable": str(prefix / "share/dpkg/tupletable"),
    }
    configuration = [argument for key, value in options.items() for argument in ("-o", key + "=" + value)]

    def run(binary, *arguments):
        return subprocess.run([binary, *configuration, *arguments], capture_output=True, text=True)

    update = run(apt_get, "update")
    (root / "update.log").write_text(update.stdout + update.stderr)
    case.assertEqual(update.returncode, 0, update.stdout + update.stderr)
    policy = run(apt_cache, "policy", *FAMILY, "systemd-hwe-hwdb")
    (root / "policy.log").write_text(policy.stdout + policy.stderr)
    case.assertEqual(policy.returncode, 0, policy.stdout + policy.stderr)
    candidates = dict(re.findall(r"(?m)^([^\s:]+):\n(?:  .*\n)*?  Candidate: (\S+)", policy.stdout))
    case.assertEqual(candidates, {**dict.fromkeys(FAMILY, REFERENCE), "systemd-hwe-hwdb": "255.1.7"})
    solver = run(apt_get, "--simulate", "--allow-downgrades", "install", "systemd-coredump")
    (root / "solver.log").write_text(solver.stdout + solver.stderr)
    case.assertEqual(solver.returncode, 0, solver.stdout + solver.stderr)
    case.assertIn("Inst libsystemd-shared [" + NEWER + "] (" + REFERENCE, solver.stdout)
    case.assertIn("Inst systemd [" + NEWER + "] (" + REFERENCE, solver.stdout)
    case.assertNotIn("Remv ", solver.stdout)
