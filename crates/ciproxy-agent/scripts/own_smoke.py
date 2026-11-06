"""Private-file clients and local observations for the own-machine live check."""

import json
import os
from pathlib import Path
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, target):
        return None


class Failure(Exception):
    """A check failed without exposing response bodies or credentials."""


class Smoke:
    def __init__(self):
        self.config = json.loads(Path(os.environ["CIPROXY_OWN_SMOKE_INPUT"]).read_text())
        self.secrets = json.loads(Path(os.environ["CIPROXY_SMOKE_CREDENTIALS"]).read_text())
        self.origin = self.config["public_url"].rstrip("/")
        self.workspace = self.config["workspace_id"]
        self.machine = self.config["machine_id"]
        self.state_file = Path(os.environ.get("CIPROXY_SMOKE_STATE", "/var/lib/ciproxy/state.json"))
        self.tag = uuid.uuid4().hex
        self.evidence = Path(".context/ciproxy-mvp") / ("own-smoke-" + self.tag)
        self.evidence.mkdir(parents=True)
        self.runs = []
        self.csrf = None
        session = self.api("GET", "/api/session")
        if session["state"] != "ready":
            raise Failure("The dashboard session is not ready")
        self.csrf = session["csrf_token"]

    def request(self, origin, method, path, headers, body=None, absent=False):
        data = None if body is None else json.dumps(body).encode()
        headers = dict(headers)
        if data is not None:
            headers["Content-Type"] = "application/json"
        request = urllib.request.Request(origin + path, data=data, headers=headers, method=method)
        try:
            with urllib.request.build_opener(NoRedirect()).open(request, timeout=60) as response:
                raw = response.read()
                return json.loads(raw) if raw else None
        except urllib.error.HTTPError as error:
            if absent and error.code == 404:
                return None
            raise Failure(f"{method} {path} returned HTTP {error.code}") from None
        except (OSError, ValueError):
            raise Failure(f"{method} {path} did not complete") from None

    def api(self, method, path, body=None, absent=False):
        value = self.secrets["session_cookie"]
        cookie = value if value.startswith("__Host-ciproxy=") else "__Host-ciproxy=" + value
        headers = {"Cookie": cookie, "Origin": self.origin}
        if method != "GET" and self.csrf:
            headers["x-ciproxy-csrf"] = self.csrf
        return self.request(self.origin, method, path, headers, body, absent)

    def machine_path(self):
        return f"/api/workspaces/{self.workspace}/machines/{self.machine}"

    def action(self, action):
        return self.api("POST", self.machine_path() + "/actions/" + action)

    def github(self, method, path, body=None):
        headers = {"Authorization": "Bearer " + self.secrets["github_token"],
                   "Accept": "application/vnd.github+json", "User-Agent": "ciproxy-own-smoke",
                   "X-GitHub-Api-Version": "2026-03-10"}
        return self.request("https://api.github.com", method, path, headers, body)

    def command(self, *arguments, capture=False):
        result = subprocess.run(arguments, capture_output=True, text=True, check=False)
        if result.returncode:
            raise Failure(f"{arguments[0]} failed with exit {result.returncode}")
        return result.stdout.strip() if capture else None

    def wait(self, check, description, timeout=None):
        deadline = time.monotonic() + (timeout or self.config.get("deadline_seconds", 600))
        while time.monotonic() < deadline:
            value = check()
            if value:
                return value
            time.sleep(1)
        raise Failure("Timed out: " + description)

    def state(self):
        return json.loads(self.state_file.read_text())

    def warm(self):
        return [vm for vm in self.state()["runners"] if vm["report"]["state"]["state"] == "warm"]

    def dispatch(self, mode="host", repository=None, cache_key=None, ref=None, cache_expectation="any", runner_label="ciproxy"):
        repository = repository or self.config["repository"]
        tag = self.tag + "-" + uuid.uuid4().hex[:8]
        workflow = self.config.get("workflow", "ciproxy-agent-smoke.yml")
        path = f"/repos/{repository}/actions/workflows/{urllib.parse.quote(workflow, safe='') }"
        self.github("POST", path + "/dispatches", {"ref": ref or self.config["ref"],
                    "inputs": {"mode": mode, "smoke_id": tag, "cache_key": cache_key or tag, "cache_expectation": cache_expectation, "runner_label": runner_label}})
        def locate():
            runs = self.github("GET", path + "/runs?per_page=30&event=workflow_dispatch")["workflow_runs"]
            return next((run["id"] for run in runs if run["display_title"] == "ciproxy agent smoke " + tag), None)
        run = self.wait(locate, "dispatched workflow appears")
        item = {"id": run, "repository": repository}
        self.runs.append(item)
        self.record("dispatch", item)
        return item

    def jobs(self, run):
        return self.github("GET", f"/repos/{run['repository']}/actions/runs/{run['id']}/jobs")["jobs"]

    def local_job(self, run):
        def locate():
            names = {job.get("runner_name", "").removeprefix("ciproxy-") for job in self.jobs(run)}
            return next((vm for vm in self.state()["runners"] if vm["report"]["runner_id"] in names
                         and vm["report"]["state"]["state"] == "running"), None)
        vm = self.wait(locate, "workflow uses this own machine")
        run["vm"] = vm["local_id"]
        run["job"] = vm["report"]["state"]["job_id"]
        self.record("local_job", {"id": run["id"], "vm": run["vm"], "job": run["job"],
                                  "slots": vm["report"]["slot_indices"]})
        return vm

    def finish(self, run):
        path = f"/repos/{run['repository']}/actions/runs/{run['id']}"
        completed = self.wait(lambda: (value if (value := self.github("GET", path))["status"] == "completed" else None), "workflow completion")
        if completed["conclusion"] != "success":
            raise Failure("Workflow did not succeed: " + str(run["id"]))
        if "vm" in run:
            self.wait(lambda: all(vm["local_id"] != run["vm"] for vm in self.state()["runners"]), "used VM disappears")
        self.record("completed", {"id": run["id"], "conclusion": completed["conclusion"]})

    def pid(self, vm):
        return self.command("systemctl", "show", "--property=MainPID", "--value",
                            "ciproxy-vm-" + vm["local_id"] + ".service", capture=True)

    def record(self, event, fields):
        with (self.evidence / "observations.jsonl").open("a") as output:
            output.write(json.dumps({"event": event, **fields}) + "\n")

    def cleanup(self):
        for run in self.runs:
            try:
                self.github("POST", f"/repos/{run['repository']}/actions/runs/{run['id']}/cancel")
            except Failure:
                pass
