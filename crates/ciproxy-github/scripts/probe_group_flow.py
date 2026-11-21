"""Live runner-group execution and cleanup. This module does not run at import."""

import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time

from probe_common import Cleanup, ProbeError, SCRATCH, emit, expect, segment
from probe_fixtures import WorkflowFixture, default_snapshot, jobs, operator
from probe_scales import ScaleService


def organization_plan(org):
    status, response = operator("/orgs/" + segment(org))
    plan = (response.get("plan") or {}).get("name") if status == 200 else None
    known = plan if plan in ("free", "team", "enterprise", "business") else "unknown"
    emit("organization_plan", http_status=status, organization=org, plan=known,
         free_proven=False, note="Free remains unproven until this execution check passes on a Free organization")
    return known


def group_snapshot(app):
    groups, page = [], 1
    while True:
        status, response = app.call(f"/orgs/{segment(app.org)}/actions/runner-groups?per_page=100&page={page}")
        expect(status, (200,), "Read organization runner groups")
        batch = response["runner_groups"]
        for row in batch:
            if row.get("visibility") == "selected":
                row["_selected_repository_ids"] = selected_repositories(app, row["id"])
        groups.extend(batch)
        if len(batch) < 100:
            return groups
        page += 1


def selected_repositories(app, group_id):
    result, page = [], 1
    while True:
        status, response = app.call(f"/orgs/{segment(app.org)}/actions/runner-groups/{group_id}/repositories?per_page=100&page={page}")
        expect(status, (200,), "Read selected-repository policy")
        batch = response["repositories"]
        result.extend(row["id"] for row in batch)
        if len(batch) < 100:
            return sorted(result)
        page += 1


def create_group(app, name, repo_id, cleanup):
    prefix = f"/orgs/{segment(app.org)}/actions/runner-groups"
    previous = group_snapshot(app)
    if any(row["name"] == name for row in previous):
        raise ProbeError("The unique probe runner-group name already exists")

    def verify_existing():
        fields = ("id", "name", "visibility", "allows_public_repositories", "restricted_to_workflows",
                  "selected_workflows", "_selected_repository_ids")
        current = {row["id"]: row for row in group_snapshot(app)}
        for row in previous:
            if row["id"] not in current:
                raise ProbeError("A pre-existing runner group disappeared during the probe")
            if any(current[row["id"]].get(key) != row.get(key) for key in fields):
                raise ProbeError("A pre-existing runner-group policy changed during the probe")
        emit("existing_runner_group_policies_unchanged", count=len(previous))

    cleanup.add("pre-existing runner-group policies", verify_existing, -10)
    group_id, rejected = None, False

    def remove():
        nonlocal group_id
        if group_id is None:
            matches = [row for row in group_snapshot(app) if row["name"] == name]
            if not matches:
                return
            if rejected or len(matches) != 1:
                raise ProbeError("Unconfirmed group name conflicts; operator inspection is required")
            group_id = int(matches[0]["id"])
            emit("created_group_recovered", group_id=group_id, name=name)
        status, current = app.call(prefix + "/" + str(group_id))
        if status == 200:
            if current.get("name") != name:
                raise ProbeError("Probe group identity changed; no group was deleted")
            status, _ = app.call(prefix + "/" + str(group_id), "DELETE")
            expect(status, (204,), "Delete probe runner group")
        else:
            expect(status, (404,), "Read probe runner group during cleanup")
        status, _ = app.call(prefix + "/" + str(group_id))
        expect(status, (404,), "Verify probe runner group is absent")

    cleanup.add("runner group " + name, remove, 30)
    emit("runner_group_create_intent", name=name, allowed_repository_id=repo_id)
    status, group = app.call(prefix, "POST", {"name": name, "visibility": "selected",
                                            "allows_public_repositories": False,
                                            "selected_repository_ids": [repo_id]})
    rejected = 400 <= status < 500
    expect(status, (201,), "Create selected-repository runner group")
    group_id = int(group["id"])
    status, selected = app.call(prefix + f"/{group_id}/repositories?per_page=100")
    expect(status, (200,), "Read selected repositories of probe runner group")
    if (group.get("name") != name or group.get("visibility") != "selected"
            or group.get("allows_public_repositories") is not False
            or [row["id"] for row in selected["repositories"]] != [repo_id]):
        raise ProbeError("GitHub did not retain the requested private selected-repository policy")
    emit("runner_group_created", group_id=group_id, name=name, allowed_repository_id=repo_id,
         visibility="selected", allows_public_repositories=False)
    return group_id


def launch_runner(launcher, configuration, cleanup):
    directory = tempfile.TemporaryDirectory(prefix="runner-", dir=SCRATCH)
    cleanup.add("temporary runner files", directory.cleanup, 0)
    root = Path(directory.name).resolve()
    (root / "home").mkdir()
    (root / "tmp").mkdir()
    environment = {key: os.environ[key] for key in ("PATH", "LANG", "LC_ALL", "RUNNER_ALLOW_RUNASROOT")
                   if key in os.environ}
    environment.update({"HOME": str(root / "home"), "TMPDIR": str(root / "tmp"),
                        "CIPROXY_GITHUB_PROBE_WORK_DIR": str(root),
                        "ACTIONS_RUNNER_INPUT_JITCONFIG": configuration})
    process = subprocess.Popen([str(Path(launcher).resolve())], cwd=root, env=environment,
                               stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                               stderr=subprocess.DEVNULL, start_new_session=True)

    def stop():
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=10)

    cleanup.add("probe runner process group", stop, 70)
    emit("runner_launcher_started", output="suppressed", jit_transport="environment")
    return process


def app_run(app, fixture):
    status, run = app.call(fixture.prefix + "/actions/runs/" + str(fixture.run_id))
    expect(status, (200,), "Read workflow run with App Actions read")
    return run


def check_jobs(app, allowed, refused, runner_id):
    allowed_job, refused_job = jobs(app, allowed), jobs(app, refused)
    if refused_job and (refused_job["status"] != "queued" or refused_job.get("runner_id") not in (None, 0)
                        or refused_job.get("runner_name")):
        raise ProbeError("The refused repository job left the queue or received a runner")
    if not allowed_job or not refused_job:
        return False
    run = app_run(app, allowed)
    if run["status"] == "completed":
        if run["conclusion"] != "success":
            raise ProbeError("The allowed repository probe job did not succeed")
        if allowed_job["status"] != "completed" or allowed_job.get("conclusion") is None:
            return False
        if allowed_job["conclusion"] != "success":
            raise ProbeError("The allowed repository probe job did not succeed")
        if not runner_id or allowed_job.get("runner_id") != runner_id:
            raise ProbeError("The allowed job did not run on the probe JIT registration")
        return True
    return False


def execute(app, args, name):
    plan = organization_plan(app.org)
    visible = app.visible_private_repositories([args.allowed_repo, args.refused_repo])
    with Cleanup() as cleanup:
        snapshots = {repo: default_snapshot(repo, cleanup) for repo in (args.allowed_repo, args.refused_repo)}
        group_id = create_group(app, name, visible[args.allowed_repo.lower()]["id"], cleanup)
        service = ScaleService(app)
        service.create(group_id, name, cleanup)
        service.open(name, cleanup)
        refused = WorkflowFixture(args.refused_repo, name, name, snapshots[args.refused_repo], cleanup)
        allowed = WorkflowFixture(args.allowed_repo, name, name, snapshots[args.allowed_repo], cleanup)
        deadline = time.monotonic() + args.timeout_seconds
        while not all(fixture.run_id for fixture in (allowed, refused)):
            for fixture in (allowed, refused):
                if fixture.run_id is None:
                    fixture.find_run()
            if time.monotonic() > deadline:
                raise ProbeError("Probe workflows did not appear before the time limit")
            time.sleep(2)
        emit("probe_runs_found", allowed_run_id=allowed.run_id, refused_run_id=refused.run_id)
        runner_id, process, succeeded_at = None, None, None
        counts = {}
        while time.monotonic() < deadline:
            message_id, rows = service.poll()
            for row in rows:
                candidate = f"{row.get('ownerName', '')}/{row.get('repositoryName', '')}".lower()
                if candidate != args.allowed_repo.lower() or str(row.get("workflowRunId")) != str(allowed.run_id):
                    raise ProbeError("The scale set received a job outside the allowed probe run")
                kind = row.get("messageType")
                counts[kind] = counts.get(kind, 0) + 1
                emit("scale_job_message", **{key: row.get(key) for key in (
                    "messageType", "jobId", "workflowRunId", "runnerRequestId", "runnerId",
                    "queueTime", "scaleSetAssignTime")})
                if kind == "JobAvailable":
                    request_id = row.get("runnerRequestId")
                    if not isinstance(request_id, int) or request_id <= 0:
                        raise ProbeError("JobAvailable lacks an explicit positive request ID")
                    service.acquire([request_id])
                if kind == "JobAssigned" and runner_id is None:
                    runner_id, configuration = service.register(name + "-runner", cleanup)
                    process = launch_runner(args.runner_launcher, configuration, cleanup)
                    del configuration
            if message_id is not None:
                service.acknowledge(message_id)
            succeeded = check_jobs(app, allowed, refused, runner_id)
            if succeeded and succeeded_at is None:
                succeeded_at = time.monotonic()
                emit("allowed_job_succeeded", run_id=allowed.run_id, runner_id=runner_id)
            if process is not None and process.poll() not in (None, 0):
                raise ProbeError("The runner launcher failed; its output was suppressed")
            if succeeded_at is not None and time.monotonic() - succeeded_at >= args.refused_seconds:
                emit("runner_group_execution_verified", allowed_repository=args.allowed_repo,
                     refused_repository=args.refused_repo, refused_run_id=refused.run_id,
                     refused_status="queued", refused_runner_id=None,
                     observation_seconds=round(time.monotonic() - succeeded_at, 1),
                     scale_message_counts=counts, organization_plan=plan, free_proven=plan == "free")
                break
        else:
            raise ProbeError("Runner execution or refused-job observation exceeded the time limit")
    emit("probe_complete", cleanup="verified", free_proven=plan == "free")
