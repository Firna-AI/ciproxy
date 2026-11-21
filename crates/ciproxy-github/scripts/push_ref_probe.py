#!/usr/bin/env python3
"""Create same-name branch/tag push runs and inspect them with the App token."""

import argparse
import time
import uuid

from probe_common import App, Cleanup, ProbeError, emit, entrypoint, expect, repository
from probe_fixtures import WorkflowFixture, checked, default_snapshot, temporary_ref
from run_job_ref_probe import read_case

PROCEDURE = """
Environment:
  CIPROXY_GITHUB_APP_PATH  Required App JSON file with id and pem. id may be a string.
  GH_TOKEN / gh auth      Operator credentials with Contents and workflow write,
                         and Actions write for fixture run cancellation/deletion.
                         App grants stay Self-hosted runners write, Metadata read,
                         and Actions read. No extra App permission is requested.

Operator procedure, from the repository root:
  1. Install Python 3, OpenSSL, and gh. Authenticate gh as the fixture operator.
  2. Install the App on one private disposable repository with an initial commit.
     The repository must allow GitHub-hosted ubuntu-latest jobs and have quota.
     Existing push workflows must not affect other systems.
  3. Export CIPROXY_GITHUB_APP_PATH to its JSON file path.
  4. Run --repo Firna-AI/ciproxy-spike. No runner launcher or Boat is required.
     The script creates a unique branch and a one-step push workflow on it.
     It waits for that branch push run before creating a same-name lightweight
     tag at the exact workflow commit. It then waits for the second push run.
  5. Read workflow_run_metadata, workflow_job_metadata, ref_comparison, and
     push_ref_fixture_verified. Optional expected refs are controlled fixture
     facts. They do not select a missing-ref fallback or a trust policy.
  6. Check every cleanup_pass record. The script cancels/deletes its runs and
     removes both unique refs. It verifies ref/run absence and the unchanged
     default-branch SHA. It changes no existing default branch or workflow file.

Both jobs use GitHub-hosted ubuntu-latest. The workflow runs only 'true', has no
checkout, and grants no token permissions. It requests no logs or artifacts.
Tag-run discovery uses the unique workflow path and commit SHA. It does not
assume that head_branch contains a tag name. Only allowlisted JSON is printed.
SIGINT/SIGTERM trigger cleanup. A network outage or SIGKILL can prevent cleanup.
Use only the safe create-intent records to inspect remaining unique resources.
"""


class PushFixture(WorkflowFixture):
    def runs(self, only_fixture=True):
        """Find tag runs even when their head_branch is null or a short name."""
        rows, page = [], 1
        while True:
            response = checked(self.prefix + f"/actions/runs?event=push&per_page=100&page={page}")
            batch = response["workflow_runs"]
            for run in batch:
                own_workflow = self.matches_workflow_path(run.get("path"))
                own_commit = self.sha is not None and run.get("head_sha") == self.sha
                if only_fixture:
                    selected = own_workflow and (self.sha is None or own_commit)
                else:
                    selected = own_workflow or own_commit or run.get("head_branch") == self.branch
                if selected:
                    rows.append(run)
            if len(batch) < 100:
                return rows
            page += 1


def wait_until(callback, deadline, label):
    while time.monotonic() < deadline:
        result = callback()
        if result is not None:
            return result
        time.sleep(2)
    raise ProbeError(label + " did not complete before the time limit")


def inspect_completed(app, fixture, run_id):
    path = fixture.prefix + "/actions/runs/" + str(run_id)
    status, run = app.call(path)
    expect(status, (200,), "Read hosted push run with App Actions read")
    if (run.get("event") != "push" or run.get("head_sha") != fixture.sha
            or not fixture.matches_workflow_path(run.get("path"))):
        raise ProbeError("The discovered run does not match the unique push workflow and commit")
    if run.get("status") != "completed":
        return None
    if run.get("conclusion") != "success":
        raise ProbeError("A hosted push fixture run failed; no runner output was requested")
    return run


def execute(app, args):
    app.visible_private_repositories([args.repo])
    name = "ciproxy-ref-push-" + uuid.uuid4().hex
    deadline = time.monotonic() + args.timeout_seconds
    with Cleanup() as cleanup:
        sha = default_snapshot(args.repo, cleanup)
        fixture = PushFixture(args.repo, name, "ubuntu-latest", sha, cleanup, include_tags=True)
        branch_run = wait_until(fixture.find_run, deadline, "Branch push run")
        branch_id = branch_run["id"]
        emit("branch_push_run_found", run_id=branch_id, branch=name, sha=fixture.sha)
        temporary_ref(args.repo, "tags/" + name, fixture.sha, cleanup)

        def tag_run():
            rows = [run for run in fixture.runs() if run["id"] != branch_id]
            if len(rows) > 1:
                raise ProbeError("The unique tag fixture produced multiple additional push runs")
            return rows[0] if rows else None

        tag_id = wait_until(tag_run, deadline, "Tag push run")["id"]
        emit("tag_push_run_found", run_id=tag_id, tag=name, sha=fixture.sha)
        for label, run_id, ref in (("same-name-branch", branch_id, "refs/heads/" + name),
                                   ("same-name-tag", tag_id, "refs/tags/" + name)):
            wait_until(lambda: inspect_completed(app, fixture, run_id), deadline, label + " hosted job")
            read_case(app, {"case": label, "repository": args.repo, "run_id": run_id, "expected_ref": ref})
        emit("push_ref_fixture_verified", repository=args.repo, branch_run_id=branch_id,
             tag_run_id=tag_id, short_name=name, sha=fixture.sha, runner="github-hosted",
             trust_policy="unselected")
    emit("probe_complete", cleanup="verified", ciproxy_runners=0, logs_read=0, artifacts_read=0)


def main():
    parser = argparse.ArgumentParser(description=__doc__, epilog=PROCEDURE,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", required=True, help="Private disposable OWNER/NAME visible to the App")
    parser.add_argument("--timeout-seconds", type=int, default=600, help="Combined run limit, default 600")
    args = parser.parse_args()
    repository(args.repo)
    if args.timeout_seconds <= 0:
        raise ProbeError("Use a positive execution time limit")
    execute(App(args.repo.split("/")[0]), args)


if __name__ == "__main__":
    entrypoint(main)
