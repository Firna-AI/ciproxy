#!/usr/bin/env python3
"""Check captured targets and label replacement in a disposable live workspace."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
from uuid import UUID

sys.dont_write_bytecode = True

from overflow_smoke.support import Live, wait
from overflow_smoke.binding_lifecycle import installation, reinstall, replace_inactive, restart


def bindings(live):
    return live.query("""SELECT COALESCE(json_agg(json_build_object(
      'id',s.id,'organization',s.organization_id,'label',s.label_id,
      'remote',s.github_id,'removed',s.remote_removed_at IS NOT NULL,
      'retired',l.deleted_at IS NOT NULL)), '[]'::json)
      FROM scale_sets s JOIN labels l ON l.workspace_id=s.workspace_id AND l.id=s.label_id
      WHERE s.workspace_id='%s'::uuid""" % live.workspace)


def runner(live, name):
    identity = str(UUID(name.removeprefix("ciproxy-")))
    return live.query("""SELECT COALESCE((SELECT json_build_object(
      'id',r.id,'organization',r.organization_id,'label',r.label_id,
      'binding',r.scale_set_id,'remote',r.registration_scale_set_id,
      'state',r.state,'own',r.machine_id IS NOT NULL,'sandbox',r.sandbox_id)
      FROM runners r WHERE r.workspace_id='%s'::uuid AND r.id='%s'::uuid),
      'null'::json)""" % (live.workspace, identity))


def observe(live):
    applied = live.query("""SELECT to_json(EXISTS(SELECT 1 FROM information_schema.columns
      WHERE table_schema='public' AND table_name='runners'
      AND column_name='registration_scale_set_id'))""")
    if not applied:
        raise ValueError("registration migration is missing")
    counts = live.query("""SELECT json_build_object(
      'captured',count(*) FILTER (WHERE registration_scale_set_id IS NOT NULL),
      'unknown_idle',count(*) FILTER (WHERE registration_scale_set_id IS NULL
        AND state IN ('starting','ready')),
      'unknown_busy',count(*) FILTER (WHERE registration_scale_set_id IS NULL AND state='busy'))
      FROM runners WHERE workspace_id='%s'::uuid""" % live.workspace)
    print("PASS: registration target column is available")
    print("Captured runners: {captured}. Unknown idle: {unknown_idle}. Unknown busy: {unknown_busy}.".format(**counts))


def baseline(live, path):
    output = Path(path)
    output.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(output, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    os.fchmod(descriptor, 0o600)
    with os.fdopen(descriptor, "w") as stream:
        json.dump({"workspace": live.workspace,
                   "organization": str(UUID(live.config["binding_organization_id"])),
                   "bindings": bindings(live), "runners": runner_targets(live)}, stream)
    print("PASS: saved binding identities in the private baseline file")


def verify_reactivation(live, path):
    previous = json.loads(Path(path).read_text())
    if previous["workspace"] != live.workspace:
        raise ValueError("baseline workspace differs")
    current = bindings(live)
    for old in previous["bindings"]:
        retained = next((row for row in current if row["id"] == old["id"]), None)
        if retained is None or retained["label"] != old["label"]:
            raise ValueError("reactivation lost a retained binding")
        if retained["retired"] and not retained["removed"]:
            tracked = live.query("""SELECT to_json(EXISTS(SELECT 1 FROM github_jobs
              WHERE workspace_id='%s'::uuid AND scale_set_id='%s'::uuid
              AND ((phase>=0 AND phase<2) OR delivered_revision<revision)))"""
                % (live.workspace, str(UUID(old["id"]))))
            if not tracked:
                raise ValueError("idle retirement still awaits reconciliation or its listener lease")
    organizations = live.api("GET", "/organizations")
    organization = next(row for row in organizations if row["id"] == previous["organization"])
    if organization["state"] != "active" or not organization["activation"]["admitted"]:
        raise ValueError("the selected organization has not resumed execution")
    current_runners = {row["id"]: row for row in runner_targets(live)}
    for old in previous["runners"]:
        if current_runners.get(old["id"]) != old:
            raise ValueError("reactivation changed a retained runner target")
    print("PASS: reactivation retains binding identities and reconciles idle retirement")


def runner_targets(live):
    return live.query("""SELECT COALESCE(json_agg(json_build_object(
      'id',id,'organization',organization_id,'label',label_id,
      'binding',scale_set_id,'remote',registration_scale_set_id)), '[]'::json)
      FROM runners WHERE workspace_id='%s'::uuid""" % live.workspace)


def replace(live, destination, restart_runtime):
    name = live.config["binding_label"]
    if name == "ciproxy":
        raise ValueError("use a disposable custom label")
    label = next(row for row in live.api("GET", "/labels") if row["label"]["name"] == name)
    settings = label["label"]
    own = destination == "own"
    if settings["allow_machines"] != own or settings["allow_boat"] == own:
        raise ValueError("the fixture label must allow only the selected destination")
    if not own:
        live.policy(live.config["smoke_limit_cents"], wait_seconds=0)
    old_run = live.dispatch(name, 300 if restart_runtime else 120)
    old_job = wait(lambda: next((job for job in live.jobs(old_run)
        if job["status"] == "in_progress" and job["runner_name"]), None))
    old = runner(live, old_job["runner_name"])
    if old is None or old["remote"] is None or old["own"] != own:
        raise ValueError("fixture runner has no captured target or used another destination")
    before = next(row for row in bindings(live) if row["id"] == old["binding"])
    if old["remote"] != before["remote"]:
        raise ValueError("fixture binding changed during the initial registration")
    if restart_runtime:
        restart(live)
        recovered = runner(live, old_job["runner_name"])
        if recovered is None or recovered["remote"] != old["remote"]:
            raise ValueError("restart lost the captured registration target")
    live.api("DELETE", "/labels/" + str(UUID(label["id"])))
    replacement = live.api("POST", "/labels", settings)
    retained = runner(live, old_job["runner_name"])
    if retained is None or retained["state"] != "busy" or retained["remote"] != old["remote"]:
        raise ValueError("label replacement changed the running registration")
    new_run = live.dispatch(name, 1)
    new_job = wait(lambda: next((job for job in live.jobs(new_run)
        if job["runner_name"]), None), seconds=600)
    new = wait(lambda: runner(live, new_job["runner_name"]))
    current = next(row for row in bindings(live) if row["id"] == new["binding"])
    if (new["label"] != replacement["id"] or new["label"] == old["label"]
            or new["binding"] == old["binding"] or new["remote"] == old["remote"]
            or new["remote"] != current["remote"] or new["own"] != own):
        raise ValueError("replacement runner did not retain its exact new target")
    for run in [old_run, new_run]:
        completed = wait(lambda: next((job for job in live.jobs(run)
            if job["status"] == "completed" and job["conclusion"]), None), seconds=600)
        if completed["conclusion"] != "success":
            raise ValueError("a fixture job failed")
    wait(lambda: next((row for row in bindings(live)
        if row["id"] == old["binding"] and row["removed"]), None))
    if not own:
        wait(lambda: live.ended(old["sandbox"]))
        wait(lambda: live.ended(new["sandbox"]))
    retained = runner(live, old_job["runner_name"])
    if retained is None or retained["remote"] != old["remote"]:
        raise ValueError("cleanup changed the original captured target")
    print("PASS: replacement uses a new target and preserves running work and retained history")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("observe", help="read migration and legacy runner counts")
    for name in ["baseline", "verify-reactivation"]:
        command = commands.add_parser(name)
        command.add_argument("state_file")
    command = commands.add_parser("replace", help="replace a disposable custom label with running work")
    command.add_argument("destination", choices=["own", "boat"])
    command.add_argument("--restart", action="store_true", help="restart the runtime while the first fixture job runs")
    for name in ["suspend", "resume", "remove", "replace-inactive", "restart"]:
        commands.add_parser(name)
    command = commands.add_parser("reinstall", help="write a private install URL and wait for browser installation")
    command.add_argument("install_file")
    command.add_argument("--open-browser", action="store_true")
    arguments = parser.parse_args()
    if not os.environ.get("CIPROXY_OVERFLOW_CREDENTIALS_FILE"):
        print("Set CIPROXY_OVERFLOW_CREDENTIALS_FILE to a private test credential file.")
        return 2
    live = None
    result = 0
    try:
        live = Live()
        observe(live)
        if arguments.command == "baseline":
            baseline(live, arguments.state_file)
        elif arguments.command == "verify-reactivation":
            verify_reactivation(live, arguments.state_file)
        elif arguments.command == "replace":
            replace(live, arguments.destination, arguments.restart)
        elif arguments.command in {"suspend", "resume", "remove"}:
            installation(live, arguments.command)
        elif arguments.command == "replace-inactive":
            replace_inactive(live)
        elif arguments.command == "reinstall":
            reinstall(live, arguments.install_file, arguments.open_browser)
        elif arguments.command == "restart":
            restart(live)
    except (OSError, ValueError, KeyError, TypeError, StopIteration, subprocess.SubprocessError):
        print("FAIL: binding validation could not finish. No credential or response body was printed.")
        result = 1
    finally:
        if live is not None:
            failures = live.restore()
            for _ in failures:
                print("FAIL: fixture cleanup needs operator attention.")
            if failures:
                result = 1
    return result


if __name__ == "__main__":
    sys.exit(main())
