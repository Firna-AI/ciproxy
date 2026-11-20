#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Run or resume a native image plan and retain each provisioner result."""

import argparse
import fcntl
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import time

from boat_build_owners import capture as capture_owners
from boat_build_report import collect, digest, save, service_inventory


def fingerprint(step):
    checksum = hashlib.sha256(json.dumps(step, sort_keys=True).encode())
    sources = step.get("sources", [step["script"]] if "script" in step else []) + step.get("inputs", [])
    for source in sources:
        path = pathlib.Path(source)
        files = sorted(path.rglob("*")) if path.is_dir() else [path]
        for file in files:
            if file.is_file():
                checksum.update(str(file).encode())
                checksum.update((digest(file) or "").encode())
    return checksum.hexdigest()


def copy_step(step):
    destination = pathlib.Path(step["destination"])
    for name in step["sources"]:
        source = pathlib.Path(name)
        if source.is_dir():
            target = destination if step.get("directory_contents") else destination / source.name
            shutil.copytree(source, target, dirs_exist_ok=True)
        else:
            target = destination / source.name if str(destination).endswith("/") or destination.is_dir() else destination
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)


def run_step(step, output):
    if step["type"] == "file":
        copy_step(step)
        return 0
    if "script" in step:
        command = (["pwsh", "-NoLogo", "-NoProfile", "-File", step["script"]]
                   if step["interpreter"] == "pwsh" else ["bash", "-e", step["script"]])
    else:
        command = ["bash", "-e", "-c", "\n".join(step["inline"])]
    load_environment = ('if [ -f /etc/profile ]; then . /etc/profile; fi; '
                        'set -a; if [ -f /etc/environment ]; then . /etc/environment; fi; '
                        'set +a; if [ -f "$HOME/.profile" ]; then . "$HOME/.profile"; fi; exec "$@"')
    command = ["bash", "-c", load_environment, "ciproxy-build", *command]
    environment = {**os.environ, "DEBIAN_FRONTEND": "noninteractive", "SUDO_USER": "user",
                   "HOME": "/root", "USER": "root", "LOGNAME": "root", **step["environment"]}
    if step.get("user") == "user":
        environment.update(HOME="/home/user", USER="user", LOGNAME="user")
        command = ["runuser", "-u", "user", "--", *command]
    return subprocess.run(command, env=environment, cwd=step["working_directory"],
                          stdout=output, stderr=subprocess.STDOUT).returncode


def execute(build_dir, resume=False, inventory=True):
    report_path = build_dir / "report.json"
    plan = json.loads((build_dir / "plan.json").read_text())
    report = json.loads(report_path.read_text()) if resume and report_path.exists() else {
        "schema": 1, "image_id": plan.get("image_id"), "provisioners": [], "status": "running"}
    previous = {step["id"]: step for step in report["provisioners"]}
    report["status"] = "running"
    for name in ("adaptations", "upstream_commit", "software_pins_sha256"):
        if name in plan:
            report[name] = plan[name]
    report["provisioners"] = []
    logs = build_dir / "logs"
    logs.mkdir(exist_ok=True)
    temporary = build_dir / "tmp"
    temporary.mkdir(exist_ok=True)
    temporary.chmod(0o1777)
    os.environ["TMPDIR"] = str(temporary)
    capture_owners(build_dir)
    if inventory and not (build_dir / "services-before.json").exists():
        save(build_dir / "services-before.json", service_inventory())
    failure = False
    for original in plan["steps"]:
        step = json.loads(json.dumps(original).replace("/opt/ciproxy-build", str(build_dir)))
        if step["type"] == "shell":
            step.setdefault("working_directory", str(temporary))
        record = {"id": step["id"], "provisioner": step["provisioner"], "type": step["type"],
                  "name": pathlib.Path(step.get("script", step["id"])).name,
                  "input_sha256": fingerprint(step)}
        if failure:
            record["status"] = "pending"
        elif step.get("skip"):
            record.update(status="skipped", reason=step["skip"])
        elif (previous.get(step["id"], {}).get("status") == "passed" and
              previous[step["id"]].get("input_sha256") == record["input_sha256"]):
            record = previous[step["id"]]
        else:
            log_path = logs / (step["id"] + ".log")
            record.update(status="running", log=str(log_path.relative_to(build_dir)))
            report["provisioners"].append(record)
            save(report_path, report)
            started = time.monotonic()
            with log_path.open("a") as output:
                try:
                    code = run_step(step, output)
                except (OSError, ValueError) as error:
                    output.write(str(error) + "\n")
                    code = 1
            record.update(exit_code=code, seconds=round(time.monotonic() - started, 3),
                          status="passed" if code == 0 else "failed")
            failure = code != 0
            print(json.dumps(record), flush=True)
            save(report_path, report)
            continue
        report["provisioners"].append(record)
        save(report_path, report)
    report["status"] = "failed" if failure else "passed"
    if inventory:
        try:
            collect(build_dir)
        except (OSError, ValueError, subprocess.CalledProcessError) as error:
            report["inventory_warning"] = str(error)
    save(report_path, report)
    return 1 if failure else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build_dir", type=pathlib.Path)
    parser.add_argument("--resume", action="store_true")
    parser.add_argument("--no-inventory", action="store_true")
    args = parser.parse_args()
    args.build_dir = args.build_dir.resolve()
    with (args.build_dir / "build.lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        return execute(args.build_dir, args.resume, not args.no_inventory)


if __name__ == "__main__":
    raise SystemExit(main())
