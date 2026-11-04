"""Operator-owned temporary refs, workflow runs, and an approved facts-pr PR."""

import base64
import time

from probe_common import ProbeError, emit, expect, repository, segment
from probe_fixtures import checked, default_snapshot, operator, temporary_ref

LABEL = "ciproxy-facts"
PR_HEAD = "facts-pr"


class Resources:
    def __init__(self, app, repo, nonce, cleanup, needs_pr=False, expected_sha=None):
        self.app, self.repo, self.nonce, self.cleanup = app, repo, nonce, cleanup
        self.prefix = "/repos/" + repository(repo)
        self.default_branch = checked(self.prefix)["default_branch"]
        self.default_sha = default_snapshot(repo, cleanup)
        if expected_sha and not self.default_sha.startswith(expected_sha):
            raise ProbeError("The default SHA differs from the operator's expected fixture SHA")
        self.intents, self.owned_runs, self.pr_number = [], set(), None
        self.pr_sha = None
        if needs_pr:
            status, _ = operator(self.prefix + "/git/ref/heads/" + PR_HEAD)
            expect(status, (404,), "The approved facts-pr head must be absent")
            if self.find_prs():
                raise ProbeError("An existing open facts-pr pull request blocks the probe")
        cleanup.add("owned workflow runs", self.remove_runs, 90)
        cleanup.add("final workflow-run sweep", self.remove_runs, 10)

    def path_matches(self, value, expected):
        path = value.split("@", 1)[0] if isinstance(value, str) else ""
        return path == expected or (path.endswith("/" + expected)
                                   and path[:-len(expected) - 1].lower() == self.repo.lower())

    def runs(self):
        rows, page = [], 1
        while True:
            response = checked(self.prefix + f"/actions/runs?per_page=100&page={page}")
            rows += response["workflow_runs"]
            if len(response["workflow_runs"]) < 100:
                return rows
            page += 1

    def fixture(self, path):
        workflow = checked(self.prefix + "/actions/workflows/" + segment(path.rsplit("/", 1)[-1]))
        if not self.path_matches(workflow.get("path"), path) or workflow.get("state") != "active":
            raise ProbeError("The required existing workflow fixture is not active")
        contents = checked(self.prefix + "/contents/" + path + "?ref=" + self.default_sha)
        if contents.get("type") != "file":
            raise ProbeError("The required workflow fixture is absent from the default SHA")
        emit("existing_fixture_verified", workflow=path, default_sha=self.default_sha)

    def intent(self, name, event, workflow, count, shas=None, branches=None):
        item = {"name": name, "event": event, "workflow": workflow, "expected": count,
                "before": {run["id"] for run in self.runs()}, "shas": set(shas or []),
                "branches": set(branches or []), "run_id": None}
        self.intents.append(item)
        return item

    def discover(self, item):
        if item["run_id"] is not None:
            status, run = operator(self.prefix + "/actions/runs/" + str(item["run_id"]))
            expect(status, (200, 404), "Read the already identified owned run")
            return run if status == 200 else None
        rows = [run for run in self.runs() if run["id"] not in item["before"]
                and run.get("event") == item["event"] and self.path_matches(run.get("path"), item["workflow"])
                and (not item["shas"] or run.get("head_sha") in item["shas"])
                and (not item["branches"] or run.get("head_branch") is None
                     or run.get("head_branch") in item["branches"])]
        if len(rows) > 1:
            raise ProbeError("Several new runs match one fixture intent; no run was selected")
        if rows:
            item["run_id"] = rows[0]["id"]
            self.owned_runs.add(rows[0]["id"])
        return rows[0] if rows else None

    def wait(self, callback, deadline, label):
        while time.monotonic() < deadline:
            result = callback()
            if result is not None:
                return result
            time.sleep(2)
        raise ProbeError(label + " was not observed before the time limit")

    def write(self, branch, path, text):
        lookup = self.prefix + "/contents/" + path + "?ref=" + segment(branch)
        status, existing = operator(lookup)
        expect(status, (200, 404), "Inspect the temporary branch file")
        body = {"message": "Prepare runner-free workflow ref probe", "branch": branch,
                "content": base64.b64encode(text.encode()).decode()}
        if status == 200:
            body["sha"] = existing["sha"]
        result = checked(self.prefix + "/contents/" + path, "PUT", body, (200, 201))
        return result["commit"]["sha"]

    def branch(self, name):
        temporary_ref(self.repo, "heads/" + name, self.default_sha, self.cleanup)

    def tag(self, name, sha):
        temporary_ref(self.repo, "tags/" + name, sha, self.cleanup)

    def dispatch(self, workflow, tag):
        checked(self.prefix + "/actions/workflows/" + segment(workflow.rsplit("/", 1)[-1]) + "/dispatches",
                "POST", {"ref": "refs/tags/" + tag}, (204,))

    def find_prs(self):
        head = segment(self.repo.split("/")[0] + ":" + PR_HEAD)
        return checked(self.prefix + "/pulls?state=open&head=" + head + "&base=" + segment(self.default_branch))

    def open_pr(self):
        self.branch(PR_HEAD)
        path = ".ciproxy-probes/" + self.nonce + ".txt"
        self.pr_sha = self.write(PR_HEAD, path, "Runner-free workflow ref probe.\n")
        self.cleanup.add("approved facts-pr pull request", self.close_pr, 85)
        emit("pull_request_create_intent", head=PR_HEAD, base=self.default_branch, head_sha=self.pr_sha)
        response = checked(self.prefix + "/pulls", "POST", {
            "title": self.nonce, "body": "Temporary runner-free metadata probe. No job will run.",
            "head": PR_HEAD, "base": self.default_branch}, (201,))
        self.pr_number = int(response["number"])
        emit("temporary_pull_request_opened", number=self.pr_number, head=PR_HEAD)

    def close_pr(self):
        if self.pr_number is None:
            rows = [row for row in self.find_prs() if row.get("head", {}).get("sha") == self.pr_sha]
            if len(rows) > 1:
                raise ProbeError("Several PRs match the owned head; no PR was closed")
            if not rows:
                return
            self.pr_number = int(rows[0]["number"])
            emit("created_pull_request_recovered", number=self.pr_number)
        path = self.prefix + "/pulls/" + str(self.pr_number)
        current = checked(path)
        if current.get("head", {}).get("ref") != PR_HEAD or current.get("head", {}).get("sha") != self.pr_sha:
            raise ProbeError("The temporary PR identity changed; no PR was closed")
        if current.get("state") != "closed":
            checked(path, "PATCH", {"state": "closed"})
        if checked(path).get("state") != "closed":
            raise ProbeError("The temporary PR is not closed")
        emit("temporary_pull_request_closed", number=self.pr_number, record_retained=True)

    def cancel(self, run_id):
        path = self.prefix + "/actions/runs/" + str(run_id)
        current = checked(path)
        if current.get("status") != "completed":
            status, _ = operator(path + "/cancel", "POST")
            expect(status, (202, 409), "Cancel owned runner-free run")
            for _ in range(30):
                if checked(path).get("status") == "completed":
                    break
                time.sleep(2)
            else:
                raise ProbeError("An owned run did not finish cancellation within 60 seconds")
        emit("owned_run_cancelled", run_id=run_id)

    def remove_runs(self):
        for item in self.intents:
            self.discover(item)
        for run_id in sorted(self.owned_runs):
            path = self.prefix + "/actions/runs/" + str(run_id)
            status, _ = operator(path)
            if status == 404:
                continue
            expect(status, (200,), "Read owned run during cleanup")
            self.cancel(run_id)
            checked(path, "DELETE", allowed=(204,))
            status, _ = operator(path)
            expect(status, (404,), "Verify owned run is absent")
