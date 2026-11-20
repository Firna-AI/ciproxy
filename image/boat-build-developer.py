#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Remove Boat entries that shadow or block the pinned image's tools."""

import argparse
import fnmatch
import hashlib
import json
import os
import pathlib
import re
import stat
import subprocess


RECORD_NAME = "provider-developer-environment.json"


def describe(path):
    try:
        mode = path.lstat().st_mode
    except FileNotFoundError:
        return {"type": "absent"}
    if stat.S_ISLNK(mode):
        return {"type": "symlink", "link_target": str(path.readlink())}
    if stat.S_ISREG(mode):
        return {"type": "file", "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
    return {"type": "other"}


def remove(root, rule):
    path = root / rule["path"].lstrip("/")
    record = {**rule, **describe(path)}
    if record["type"] == "absent":
        record["status"] = "absent"
    elif record["type"] in ("file", "symlink"):
        path.unlink()
        record["status"] = "removed"
    else:
        record.update(status="kept", warning=rule["path"] + " is not a file or link. It stays unchanged.")
    return record


def query(arguments, program="dpkg-query"):
    try:
        return subprocess.run([program, *arguments], capture_output=True, text=True,
                              env={**os.environ, "LC_ALL": "C"})
    except OSError:
        return None


def rewrite(root, rule, transform):
    path = root / rule["path"].lstrip("/")
    if not path.exists():
        return {**rule, "status": "absent"}
    before = path.read_bytes()
    after = transform(before)
    if before != after:
        path.write_bytes(after)
    return {**rule, "status": "changed" if before != after else "unchanged",
            "before_sha256": hashlib.sha256(before).hexdigest(),
            "after_sha256": hashlib.sha256(after).hexdigest()}


def environment(root, rule):
    changes = []
    blocked = {value.encode() for value in rule["path_elements"]}

    def clean(content):
        result = []
        for line in content.splitlines(keepends=True):
            assignment = re.match(rb"[ \t]*([A-Za-z_][A-Za-z_0-9]*)[ \t]*=", line)
            name = assignment[1].decode() if assignment else ""
            if name in rule["variables"]:
                changes.append({"name": name, "action": "removed-variable"})
                continue
            if name == "PATH":
                match = re.fullmatch(rb'''([ \t]*PATH[ \t]*=[ \t]*)(["']?)(.*?)(\2)([ \t]*)(\r?\n)?''', line)
                if match:
                    elements = match[3].split(b":")
                    removed = [item for item in rule["path_elements"] if item.encode() in elements]
                    kept = [item for item in elements if item not in blocked]
                    line = match[1] + match[2] + b":".join(kept) + match[4] + match[5] + (match[6] or b"")
                    if removed:
                        changes.append({"name": name, "action": "removed-path-elements", "elements": removed})
            result.append(line)
        return b"".join(result)

    record = rewrite(root, rule, clean)
    record["changes"] = changes
    return record


def profile_script(root, rule):
    path = root / rule["path"].lstrip("/")
    if not path.exists() and not path.is_symlink():
        return {**rule, "status": "absent"}
    owner = query(["-S", rule["path"]])
    if owner is None or owner.returncode not in (0, 1):
        return {**rule, "status": "kept", "warning": "Cannot read package ownership for " + rule["path"] + "."}
    if owner.returncode == 0:
        return {**rule, "status": "kept-package-owned", "package": owner.stdout.strip()}
    return remove(root, rule)


def bash_conffile(root, rule):
    block = rule["remove_block"].encode()
    record = rewrite(root, rule, lambda content: content.replace(block, b"", 1))
    record["removed_blocks"] = 1 if record["status"] == "changed" else 0
    if record["status"] == "absent":
        return record
    answer = query(["-W", "-f=${Conffiles}", rule["package"]])
    expected = None
    if answer and answer.returncode == 0:
        for line in answer.stdout.splitlines():
            parts = line.split()
            if len(parts) >= 2 and parts[0] == rule["path"]:
                expected = parts[1]
                break
    actual = hashlib.md5((root / rule["path"].lstrip("/")).read_bytes()).hexdigest()
    record.update(expected_md5=expected, actual_md5=actual, verified_md5=expected == actual)
    if expected != actual:
        record["warning"] = rule["path"] + " does not match the installed package's conffile md5."
    return record


def retain(root, rule):
    details = describe(root / rule["path"].lstrip("/"))
    return {**rule, **details, "status": "absent" if details["type"] == "absent" else "kept"}


def package_versions():
    answer = query(["-W", "-f=${binary:Package}\t${Version}\t${db:Status-Status}\n"])
    if answer is None or answer.returncode:
        return None
    packages = {}
    for line in answer.stdout.splitlines():
        fields = line.split("\t")
        if len(fields) != 3:
            return None
        name, version, status = fields
        if status in ("installed", "config-files"):
            packages[name] = version
    return packages


def package_groups(rules):
    records = []
    action = "purge"
    for rule in rules:
        record = {**rule, "action": action, "status": "absent", "packages": []}
        records.append(record)
        before = package_versions()
        tool = rule["pinned_tool"]
        if before is None:
            record.update(status="kept", warning="Cannot read package versions for " + tool + ".")
            continue
        names = sorted(name for name in before if any(
            fnmatch.fnmatchcase(name.partition(":")[0], pattern) for pattern in rule["patterns"]))
        if not names:
            continue
        record.update(status="kept", packages=[
            {"name": name, "version": before[name], "status": "kept"} for name in names])
        arguments = ["--no-auto-remove", action, "--", *names]
        simulation = query(["--simulate", *arguments], "apt-get")
        record["simulation_exit_code"] = simulation.returncode if simulation is not None else None
        if simulation is None:
            record["warning"] = "Cannot simulate removal of " + tool + " packages."
            continue
        planned = sorted({line.split()[1] for line in simulation.stdout.splitlines()
                          if line.startswith(("Remv ", "Purg ")) and len(line.split()) > 1})
        record["simulated_removals"] = planned
        extra = sorted(set(planned) - set(names))
        record["extra_removals"] = extra
        if extra:
            record["warning"] = "Kept " + tool + " packages because apt would also remove: " + ", ".join(extra) + "."
            continue
        if simulation.returncode:
            record["warning"] = "Cannot simulate removal of " + tool + " packages."
            continue
        if not planned:
            record["warning"] = "Apt reported no package removal plan for " + tool + "."
            continue
        removal = query(["-y", *arguments], "apt-get")
        record["removal_exit_code"] = removal.returncode if removal is not None else None
        remaining = package_versions()
        for package in record["packages"]:
            package["status"] = ("unknown" if remaining is None else
                                 "kept" if package["name"] in remaining else "removed")
        if remaining is None:
            record["status"] = "unknown"
        elif all(package["status"] == "removed" for package in record["packages"]):
            record["status"] = "removed"
        if removal is None or removal.returncode or record["status"] != "removed":
            record["warning"] = "Could not complete removal of " + tool + " packages."
    return records


def cleanup(root, policy):
    record_path = root / "opt/ciproxy-build" / RECORD_NAME
    if record_path.exists():
        return
    definition = json.loads(policy.read_text())
    commands = []
    for group in definition["commands"]:
        selection = {key: group[key] for key in ("pinned_tool", "install_path", "reason")}
        commands.extend(remove(root, {**selection, "path": "/usr/local/bin/" + name, "name": name})
                        for name in group["names"])
    rules = definition["settings"]
    settings = [environment(root, rules["environment"])]
    settings.extend(profile_script(root, rule) for rule in rules["profile_scripts"])
    settings.append(bash_conffile(root, rules["bash_conffile"]))
    settings.extend(retain(root, rule) for rule in definition["kept_settings"])
    extras = [retain(root, {"path": "/usr/local/bin/" + name, "name": name, "reason": group["reason"]})
              for group in definition["kept_extras"] for name in group["names"]]
    packages = package_groups(definition["packages"])
    kept = {}
    for folder in ("/usr/local/bin", "/usr/local/sbin"):
        path = root / folder.lstrip("/")
        kept[folder] = sorted(item.name for item in path.iterdir()) if path.is_dir() else []
    record = {"commands": commands, "kept_entries": kept, "kept_extras": extras, "settings": settings,
              "packages": packages,
              "warnings": [item["warning"] for item in commands + settings + packages if "warning" in item]}
    record_path.parent.mkdir(parents=True, exist_ok=True)
    with record_path.open("x") as output:
        output.write(json.dumps(record, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=pathlib.Path, default=pathlib.Path("/"))
    parser.add_argument("--policy", type=pathlib.Path, default=pathlib.Path(__file__).with_name("boat-developer-tools.json"))
    args = parser.parse_args()
    cleanup(args.root, args.policy)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
