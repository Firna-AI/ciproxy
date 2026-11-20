# SPDX-License-Identifier: Apache-2.0
"""Keep native build inventory and changed running service records."""

import hashlib
import json
import pathlib
import subprocess

from boat_build_owners import RECORD_NAME
from apt_pin_report import compare_pins


BUILD_UNIT = "ciproxy-image-build.service"


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(".new")
    temporary.write_text(json.dumps(value, indent=2) + "\n")
    temporary.replace(path)


def digest(path):
    try:
        with pathlib.Path(path).open("rb") as source:
            checksum = hashlib.file_digest(source, "sha256") if hasattr(hashlib, "file_digest") else None
            if checksum is None:
                checksum = hashlib.sha256()
                for chunk in iter(lambda: source.read(1024 * 1024), b""):
                    checksum.update(chunk)
            return checksum.hexdigest()
    except (FileNotFoundError, PermissionError, IsADirectoryError):
        return None


def service_inventory():
    result = subprocess.run(["systemctl", "list-units", "--type=service", "--state=running",
                             "--plain", "--no-legend", "--no-pager"], capture_output=True, text=True)
    records = {}
    if result.returncode:
        return records
    for line in result.stdout.splitlines():
        unit = line.split()[0]
        if unit == BUILD_UNIT:
            continue
        details = subprocess.run(["systemctl", "show", unit, "--property=MainPID,FragmentPath"],
                                 capture_output=True, text=True, check=True)
        fields = dict(row.split("=", 1) for row in details.stdout.splitlines())
        executable = pathlib.Path("/proc") / fields.get("MainPID", "0") / "exe"
        try:
            path = str(executable.readlink()).removesuffix(" (deleted)")
        except FileNotFoundError:
            path = ""
        fragment = fields.get("FragmentPath", "")
        records[unit] = {"executable": path, "executable_sha256": digest(executable),
                         "unit": fragment, "unit_sha256": digest(fragment)}
    return records


def changed_services(before):
    changed = {"docker.service", "containerd.service"}
    for unit, record in before.items():
        if unit == BUILD_UNIT:
            continue
        if (digest(record["executable"]) != record["executable_sha256"] or
                digest(record["unit"]) != record["unit_sha256"]):
            changed.add(unit)
    return sorted(changed)


def flatten(value, prefix=""):
    result = {}
    if isinstance(value, dict):
        name = value.get("Title", value.get("ToolName", ""))
        path = prefix + "/" + name
        for key in ("Version", "Versions", "Rows"):
            if key in value:
                result[path] = value[key]
        children = value.get("Children", [])
        children = [children] if isinstance(children, dict) else children
        for child in children:
            result.update(flatten(child, path))
    return result


def compare_inventory(reference, installed):
    expected, actual = flatten(reference), flatten(installed)
    differences = [{"tool": name, "reference": expected.get(name), "installed": actual.get(name)}
                   for name in sorted(expected.keys() | actual.keys())
                   if expected.get(name) != actual.get(name)]
    return {"differences_fail_build": False, "difference_count": len(differences),
            "differences": differences}


def collect(build_dir, image_dir=pathlib.Path("/imagegeneration"),
            restart_file=pathlib.Path("/etc/ciproxy/boat-restart-services"), system_root=pathlib.Path("/")):
    artifacts = build_dir / "reports"
    artifacts.mkdir(exist_ok=True)
    owners = build_dir / RECORD_NAME
    developer = build_dir / "provider-developer-environment.json"
    pin_differences = compare_pins(system_root)
    packages = subprocess.run(["dpkg-query", "-W", "-f=${Package}\t${Version}\n"],
                              capture_output=True, text=True)
    save(artifacts / "installed-packages.json", {
        "exit_code": packages.returncode,
        "packages": dict(row.split("\t", 1) for row in packages.stdout.splitlines() if "\t" in row)})
    save(artifacts / "installed-inventory.json", {
        "packages_exit_code": packages.returncode,
        "packages": dict(row.split("\t", 1) for row in packages.stdout.splitlines() if "\t" in row),
        "software_report_available": (image_dir / "software-report.json").exists(),
        "provider_docker_gid": (pathlib.Path("/etc/ciproxy/provider-docker-gid").read_text().strip()
                                if pathlib.Path("/etc/ciproxy/provider-docker-gid").exists() else None),
        "provider_directory_owners": json.loads(owners.read_text()) if owners.exists() else None,
        "provider_developer_environment": json.loads(developer.read_text()) if developer.exists() else None,
        "apt_pin_differences": pin_differences})
    report = image_dir / "software-report.json"
    reference = build_dir / "reference-software.json"
    if report.exists():
        installed = json.loads(report.read_text())
        save(artifacts / "software-report.json", installed)
        if reference.exists():
            differences = compare_inventory(json.loads(reference.read_text()), installed)
            differences["apt_pin_differences"] = pin_differences
            save(artifacts / "inventory-differences.json", differences)
    elif reference.exists():
        differences = compare_inventory(json.loads(reference.read_text()), {})
        differences["software_report_available"] = False
        differences["reason"] = "Provisioning stopped before the upstream software report was available."
        differences["apt_pin_differences"] = pin_differences
        save(artifacts / "inventory-differences.json", differences)
    before = build_dir / "services-before.json"
    if before.exists():
        services = changed_services(json.loads(before.read_text()))
        restart_file.parent.mkdir(parents=True, exist_ok=True)
        restart_file.write_text("\n".join(services) + "\n")
        save(artifacts / "changed-services.json", {"restart_services": services})
