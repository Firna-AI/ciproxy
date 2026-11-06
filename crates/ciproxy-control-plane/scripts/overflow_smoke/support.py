"""Private live smoke inputs and bounded API and database observations."""

import json
import os
from pathlib import Path
import subprocess
import time
from urllib.error import HTTPError
from urllib.parse import quote, urlparse
from urllib.request import HTTPRedirectHandler, Request, build_opener
from uuid import UUID, uuid4


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, new_url):
        return None


class Live:
    def __init__(self):
        self.config = json.loads(Path(os.environ["CIPROXY_OVERFLOW_CREDENTIALS_FILE"]).read_text())
        self.origin = self.config["base_url"].rstrip("/")
        if urlparse(self.origin).scheme != "https":
            raise ValueError("HTTPS required")
        self.workspace = str(UUID(self.config["workspace_id"]))
        self.opener = build_opener(NoRedirect())
        self.runs = []
        self.markers = []
        self.policy_attempted = False
        self.state_file = Path(".context/ciproxy-mvp-m15/live") / (uuid4().hex + ".json")
        self.state_file.parent.mkdir(parents=True, exist_ok=True)
        self.save()
        self.baseline = {row["delivery"] for row in self.records()}
        self.original = self.api("GET", "/overflow")
        self.save()

    def request(self, method, url, headers, body=None, missing=False):
        data = None if body is None else json.dumps(body).encode()
        request = Request(url, data=data, method=method,
                          headers={**headers, "Content-Type": "application/json"})
        try:
            with self.opener.open(request, timeout=30) as response:
                raw = response.read(2_000_001)
                if len(raw) > 2_000_000:
                    raise ValueError("response too large")
                return json.loads(raw) if raw else None
        except HTTPError as error:
            if missing and error.code == 404:
                return None
            raise ValueError("request failed") from None

    def api(self, method, route, body=None):
        return self.request(method, self.origin + "/api/workspaces/" + self.workspace + route,
                            {"Cookie": "__Host-ciproxy=" + self.config["session_cookie"],
                             "Origin": self.origin,
                             "X-Ciproxy-CSRF": self.config["csrf_token"]}, body)

    def github(self, method, route, body=None):
        repository = self.config["repository"]
        if len(repository.split("/")) != 2:
            raise ValueError("repository must be owner/name")
        return self.request(method, "https://api.github.com/repos/" + repository + route,
                            {"Authorization": "Bearer " + self.config["github_token"],
                             "Accept": "application/vnd.github+json"}, body)

    def boat(self, method, sandbox, missing=False):
        return self.request(method, "https://boat.dev/api/v1/sandboxes/" + quote(sandbox, safe=""),
                            {"Authorization": "Bearer " + self.config["boat_api_key"],
                             "X-Boat-Org": self.config.get("boat_wallet", "personal")},
                            missing=missing)

    def records(self):
        sql = """SELECT COALESCE(json_agg(json_build_object(
          'delivery',delivery_id,'runner',runner_id,
          'sandbox',record_json::jsonb->'sandbox',
          'phase',record_json::jsonb->>'phase',
          'dispatched',record_json::jsonb ? 'size',
          'deployment',record_json::jsonb->>'deployment')), '[]'::json)
          FROM overflow_starts WHERE workspace_id='%s'::uuid""" % self.workspace
        return self.query(sql)

    def query(self, sql):
        env = dict(os.environ, PGDATABASE=self.config["database_name"])
        result = subprocess.run(["psql", "--no-psqlrc", "--no-align", "--tuples-only", "--command", sql],
                                env=env, capture_output=True, text=True, check=False, timeout=30)
        if result.returncode:
            raise ValueError("database observation failed")
        return json.loads(result.stdout)

    def created(self):
        rows = [row for row in self.records() if row["delivery"] not in self.baseline]
        if any(row["deployment"] != self.origin + "/" + self.workspace for row in rows):
            raise ValueError("deployment mark mismatch")
        return rows

    def dispatch(self, label, seconds):
        marker = uuid4().hex
        self.markers.append(marker)
        self.save()
        workflow = quote(str(self.config["workflow_id"]), safe="")
        route = "/actions/workflows/" + workflow
        self.github("POST", route + "/dispatches", {
            "ref": self.config["ref"],
            "inputs": {"marker": marker, "label": label, "hold_seconds": str(seconds)},
        })

        def find():
            runs = self.github("GET", route + "/runs?event=workflow_dispatch&per_page=100")["workflow_runs"]
            return next((run for run in runs if run["display_title"] == "ciproxy-smoke-" + marker), None)

        run = wait(find)
        self.runs.append(run["id"])
        self.save()
        return run["id"]

    def jobs(self, run):
        return self.github("GET", "/actions/runs/" + str(run) + "/jobs")["jobs"]

    def policy(self, limit, wait_seconds=30):
        self.policy_attempted = True
        self.api("PUT", "/overflow", {"enabled": True, "wait_seconds": wait_seconds,
                                      "spend_limit_cents": limit})

    def save(self):
        original = getattr(self, "original", None)
        policy = None if original is None else {
            key: original[key]
            for key in ("enabled", "wait_seconds", "spend_limit_cents")
        }
        self.state_file.write_text(json.dumps({
            "markers": self.markers, "runs": self.runs, "original_policy": policy,
        }))
        self.state_file.chmod(0o600)

    def own_runner(self, name):
        runner = str(UUID(name.removeprefix("ciproxy-")))
        return self.query("SELECT to_json(EXISTS(SELECT 1 FROM runners WHERE workspace_id='%s'::uuid AND id='%s'::uuid AND machine_id IS NOT NULL))" % (self.workspace, runner))

    def waited(self, row):
        runner = str(UUID(row["runner"]))
        sql = """SELECT COALESCE((SELECT to_json(EXTRACT(EPOCH FROM
          ((s.record_json::jsonb->>'starts_at')::timestamptz-j.assigned_at)))
          FROM overflow_starts s JOIN runners r ON r.id=s.runner_id AND r.workspace_id=s.workspace_id
          JOIN github_jobs j ON j.workspace_id=r.workspace_id AND j.organization_id=r.organization_id
            AND j.scale_set_id=r.scale_set_id AND (j.body->>'runner_github_id')::bigint=r.github_id
          WHERE s.workspace_id='%s'::uuid AND s.runner_id='%s'::uuid LIMIT 1), 'null'::json)""" % (self.workspace, runner)
        return self.query(sql)

    def ended(self, sandbox):
        result = self.boat("GET", sandbox, missing=True)
        if result is None:
            return True
        if result.get("ok") is not True or result.get("type") != "sandbox.info":
            raise ValueError("invalid sandbox observation")
        value = result["sandbox"]
        if value["id"] != sandbox:
            raise ValueError("sandbox identity mismatch")
        return value["state"] in {"archived", "cancelled"}

    def destroy(self, sandbox):
        operation = None

        def delete():
            nonlocal operation
            result = self.boat("DELETE", sandbox, missing=True)
            if result is None:
                return self.ended(sandbox)
            value = result["operation"]
            if result.get("ok") is not True or result.get("type") not in {"deletion.operation", "sandbox.deleting"}:
                raise ValueError("invalid deletion observation")
            if value["kind"] != "sandbox" or value["targetId"] != sandbox:
                raise ValueError("deletion identity mismatch")
            if operation is not None and operation != value["id"]:
                raise ValueError("deletion operation changed")
            operation = value["id"]
            if value["status"] not in {"pending", "processing", "blocked", "completed"}:
                raise ValueError("unknown deletion status")
            return value["status"] == "completed"

        wait(delete, seconds=90)
        wait(lambda: self.ended(sandbox), seconds=90)

    def restore(self):
        if not self.policy_attempted and not self.markers:
            return []
        failures = []
        try:
            self.api("PUT", "/overflow", {"enabled": False,
                "wait_seconds": self.original["wait_seconds"],
                "spend_limit_cents": self.original["spend_limit_cents"]})
        except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError):
            failures.append("disable new Boat dispatch")
        if self.markers:
            try:
                workflow = quote(str(self.config["workflow_id"]), safe="")
                runs = self.github("GET", "/actions/workflows/" + workflow + "/runs?event=workflow_dispatch&per_page=100")["workflow_runs"]
                found = [run for run in runs if run["display_title"] in {"ciproxy-smoke-" + marker for marker in self.markers}]
                if len(found) != len(self.markers):
                    raise ValueError("a dispatched workflow needs reconciliation")
                for run in found:
                    route = "/actions/runs/" + str(run["id"])
                    if run["status"] != "completed":
                        self.github("POST", route + "/cancel", {})
                    wait(lambda: self.github("GET", route)["status"] == "completed", seconds=90)
            except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError):
                failures.append("workflow cancellation or lost-dispatch recovery")
            try:
                for row in self.created():
                    if row["sandbox"] is not None:
                        self.destroy(sandbox_id(row))
            except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError):
                failures.append("sandbox cleanup")
        if failures:
            failures.append("settings restoration remains held until cleanup succeeds")
        else:
            try:
                self.api("PUT", "/overflow", {"enabled": self.original["enabled"],
                    "wait_seconds": self.original["wait_seconds"],
                    "spend_limit_cents": self.original["spend_limit_cents"]})
            except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError):
                failures.append("settings restoration")
        return failures


def sandbox_id(row):
    return row["sandbox"]["id"]


def wait(check, seconds=240):
    until = time.monotonic() + seconds
    while time.monotonic() < until:
        result = check()
        if result:
            return result
        time.sleep(1)
    raise ValueError("observation deadline reached")
