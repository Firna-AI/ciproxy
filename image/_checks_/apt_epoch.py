# SPDX-License-Identifier: Apache-2.0
"""Require the rendered Git pin to match the repository's Debian epoch."""

from apt_pin_report import stanzas
from _checks_.apt_fixture import repository
from _checks_.recipe_fixture import rendered


def check_epoch(case, workspace):
    _, upstream, _ = rendered(workspace)
    text = (upstream / "images/ubuntu/scripts/build/configure-apt.sh").read_text()
    preferences = text.split("<<'CIPROXY_PINS'\n", 1)[1].split("\nCIPROXY_PINS", 1)[0]
    rule = next(item for item in stanzas(preferences) if "git" in item["Package"].split()
                or "src:git" in item["Package"].split())
    declared = rule["Pin"].removeprefix("version ").rstrip("*")
    wanted = "1:" + declared.split(":", 1)[-1]
    newer = "1:9999.0.0"
    records = []
    for name in ("git", "git-man"):
        for version in (wanted, newer):
            record = f"Package: {name}\nSource: git\nVersion: {version}\nArchitecture: amd64\n"
            if name == "git":
                record += f"Depends: git-man (= {version})\n"
            records.append(record + "Description: Epoch fixture\nFilename: git.deb\nSize: 1\n")
    apt = repository(case, workspace, "\n".join(records), preferences)
    update = apt.run(apt.apt_get, "update")
    case.assertEqual(update.returncode, 0, update.stderr)
    solver = apt.run(apt.apt_get, "--simulate", "install", "git")
    case.assertEqual(solver.returncode, 0, solver.stdout + solver.stderr)
    for name in ("git", "git-man"):
        case.assertIn("Inst " + name + " (" + wanted, solver.stdout)
    case.assertNotIn(newer, solver.stdout)
