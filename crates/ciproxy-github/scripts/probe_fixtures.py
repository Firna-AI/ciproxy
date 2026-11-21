"""Operator-authenticated temporary refs, workflows, and run cleanup."""

import base64
import json
import os
import re
import subprocess
import time

from probe_common import ProbeError, VERSION, emit, expect, repository, segment


def operator(path, method="GET", body=None):
    command = ["gh", "api", "--hostname", "github.com", "--include", "--method", method,
               "--header", "Accept: application/vnd.github+json",
               "--header", "X-GitHub-Api-Version: " + VERSION]
    if body is not None:
        command += ["--input", "-"]
    command.append(path)
    environment = dict(os.environ)
    environment.pop("GH_DEBUG", None)
    environment["GH_PROMPT_DISABLED"] = "1"
    try:
        result = subprocess.run(command, input=None if body is None else json.dumps(body),
                                text=True, capture_output=True, env=environment, check=False)
    except OSError:
        raise ProbeError("Cannot run gh; install it and use operator authentication") from None
    output = result.stdout.replace("\r\n", "\n")
    match = re.match(r"HTTP/\S+\s+(\d+)", output)
    if not match:
        raise ProbeError("gh failed before an HTTP response; stdout and stderr withheld")
    status = int(match.group(1))
    if status >= 300 or status == 204:
        return status, None
    if result.returncode:
        raise ProbeError("gh failed after an HTTP response; details withheld")
    raw = output.partition("\n\n")[2].strip()
    try:
        return status, json.loads(raw) if raw else None
    except ValueError:
        raise ProbeError("gh returned non-JSON output; response withheld") from None


def checked(path, method="GET", body=None, allowed=(200,)):
    status, result = operator(path, method, body)
    expect(status, allowed, "Operator GitHub request")
    return result


def default_snapshot(repo, cleanup):
    prefix = "/repos/" + repository(repo)
    metadata = checked(prefix)
    if metadata.get("private") is not True:
        raise ProbeError("Operator fixture repository must be private")
    branch = metadata["default_branch"]
    path = prefix + "/git/ref/heads/" + segment(branch)
    sha = checked(path)["object"]["sha"]

    def verify():
        if checked(path)["object"]["sha"] != sha:
            raise ProbeError("The fixture default branch changed during the probe")
        emit("default_branch_unchanged", repository=repo, branch=branch, sha=sha)

    cleanup.add(repo + " default branch snapshot", verify, -10)
    return sha


def temporary_ref(repo, ref, sha, cleanup, prepare=None):
    prefix = "/repos/" + repository(repo)
    path = prefix + "/git/ref/" + ref
    status, _ = operator(path)
    expect(status, (404,), "Verify temporary ref does not exist")
    state = "intent"

    def remove():
        status, existing = operator(path)
        if status == 200:
            if state != "confirmed":
                if state == "rejected" or existing.get("object", {}).get("sha") != sha:
                    raise ProbeError("Unconfirmed ref identity conflicts; operator inspection is required")
                emit("temporary_ref_recovered", repository=repo, ref="refs/" + ref, sha=sha)
            if existing.get("ref") != "refs/" + ref:
                raise ProbeError("Temporary ref identity changed; no ref was deleted")
            checked(prefix + "/git/refs/" + ref, "DELETE", allowed=(204,))
        else:
            expect(status, (404,), "Find temporary ref during cleanup")
        status, _ = operator(path)
        expect(status, (404,), "Verify temporary ref is absent")

    cleanup.add(repo + " refs/" + ref, remove, 20)
    if prepare is not None:
        prepare()
    emit("temporary_ref_create_intent", repository=repo, ref="refs/" + ref, sha=sha)
    status, _ = operator(prefix + "/git/refs", "POST", {"ref": "refs/" + ref, "sha": sha})
    if 400 <= status < 500:
        state = "rejected"
    expect(status, (201,), "Create temporary ref")
    state = "confirmed"
    emit("temporary_ref_created", repository=repo, ref="refs/" + ref)


class WorkflowFixture:
    def __init__(self, repo, name, label, sha, cleanup, include_tags=False):
        self.repo = repo
        self.branch = name
        self.workflow = f".github/workflows/{name}.yml"
        self.prefix = "/repos/" + repository(repo)
        self.sha = None
        self.run_id = None
        def prepare():
            cleanup.add(repo + " probe workflow runs", self.remove_runs, 80)
            cleanup.add(repo + " final workflow-run sweep", self.remove_runs, 10)

        temporary_ref(repo, "heads/" + name, sha, cleanup, prepare)
        tag_trigger = f"    tags: [{json.dumps(name)}]\n" if include_tags else ""
        content = (
            f"name: {name}\n"
            f"on:\n  push:\n    branches: [{json.dumps(name)}]\n"
            + tag_trigger +
            "permissions: {}\n"
            "jobs:\n  probe:\n"
            f"    runs-on: {json.dumps(label)}\n"
            "    timeout-minutes: 3\n"
            "    steps:\n      - name: Confirm runner access\n        run: 'true'\n"
        )
        result = checked(self.prefix + "/contents/" + self.workflow, "PUT", {
            "message": "Run isolated ciproxy access probe", "branch": self.branch,
            "content": base64.b64encode(content.encode()).decode()}, (201,))
        self.sha = result["commit"]["sha"]
        emit("workflow_fixture_created", repository=repo, branch=name, workflow=self.workflow)

    def matches_workflow_path(self, value):
        if not isinstance(value, str):
            return False
        path = value.split("@", 1)[0]
        if path == self.workflow:
            return True
        suffix = "/" + self.workflow
        return path.endswith(suffix) and path[:-len(suffix)].lower() == self.repo.lower()

    def runs(self, only_fixture=True):
        rows = []
        page = 1
        while True:
            response = checked(self.prefix + "/actions/runs?branch=" + segment(self.branch)
                               + f"&event=push&per_page=100&page={page}")
            batch = response["workflow_runs"]
            rows += [run for run in batch if run.get("head_branch") == self.branch
                     and (not only_fixture or (self.matches_workflow_path(run.get("path"))
                                               and (self.sha is None or run.get("head_sha") == self.sha)))]
            if len(batch) < 100:
                return rows
            page += 1

    def find_run(self):
        rows = self.runs()
        if len(rows) > 1:
            raise ProbeError("The unique workflow fixture produced multiple runs")
        if rows:
            self.run_id = rows[0]["id"]
        return rows[0] if rows else None

    def remove_runs(self):
        rows = self.runs(False)
        for run in rows:
            path = self.prefix + "/actions/runs/" + str(run["id"])
            if run.get("status") != "completed":
                status, _ = operator(path + "/cancel", "POST")
                expect(status, (202, 409), "Cancel probe workflow run")
                for _ in range(30):
                    current = checked(path)
                    if current["status"] == "completed":
                        break
                    time.sleep(2)
                else:
                    raise ProbeError("Probe run did not finish cancellation within 60 seconds")
            checked(path, "DELETE", allowed=(204,))
            status, _ = operator(path)
            expect(status, (404,), "Verify probe workflow run is absent")
        if self.runs(False):
            raise ProbeError("A probe workflow run remains after cleanup")


def jobs(app, fixture):
    path = fixture.prefix + "/actions/runs/" + str(fixture.run_id) + "/jobs?per_page=100"
    status, response = app.call(path)
    expect(status, (200,), "Read fixture jobs with App Actions read")
    rows = response["jobs"]
    if len(rows) > 1:
        raise ProbeError("The probe workflow must contain exactly one job")
    return rows[0] if rows else None
