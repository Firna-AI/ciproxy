"""Independent Actions service calls, based on actions/scaleset client.go."""

import json
import time
import urllib.parse

from probe_common import API, ProbeError, emit, expect, request, segment, trusted_url

SCALE_PATH = "_apis/runtime/runnerscalesets"
RUNNER_PATH = "_apis/distributedtask/pools/0/agents"


class ScaleService:
    def __init__(self, app):
        self.app = app
        status, registration = app.call(
            f"/orgs/{segment(app.org)}/actions/runners/registration-token", "POST", {})
        expect(status, (201,), "Create runner registration token")
        for attempt in range(6):
            status, connection = request(
                API + "/actions/runner-registration", "RemoteAuth " + registration["token"],
                "POST", {"url": "https://github.com/" + app.org, "runner_event": "register"})
            if status not in (401, 403) or attempt == 5:
                break
            time.sleep(2)
        expect(status, (200,), "Discover Actions service admin connection")
        self.url = trusted_url(connection["url"])
        self.authorization = "Bearer " + connection["token"]
        self.scale_id = None
        self.scale_name = None
        self.group_id = None
        self.scale_rejected = False
        self.session = None
        self.last_message = None

    def call(self, path, method="GET", body=None, authorization=None):
        url = self.url.rstrip("/") + "/" + path.lstrip("/")
        url += ("&" if "?" in url else "?") + "api-version=6.0-preview"
        return request(url, authorization or self.authorization, method, body)

    def create(self, group_id, name, cleanup):
        status, previous = self.call(SCALE_PATH + "?" + urllib.parse.urlencode(
            {"runnerGroupId": group_id, "name": name}))
        expect(status, (200,), "Verify unique scale-set name")
        if previous.get("count") != 0:
            raise ProbeError("The unique probe scale-set name already exists")
        self.scale_name, self.group_id = name, group_id
        cleanup.add("runner scale set " + name, self.remove_scale, 40)
        emit("scale_set_create_intent", name=name, group_id=group_id)
        status, scale = self.call(SCALE_PATH, "POST", {
            "name": name, "runnerGroupId": group_id,
            "labels": [{"name": name, "type": "System"}],
            "RunnerSetting": {"disableUpdate": True}})
        self.scale_rejected = 400 <= status < 500
        expect(status, (200,), "Create runner scale set")
        self.scale_id = int(scale["id"])
        if scale.get("name") != name or scale.get("runnerGroupId") != group_id:
            raise ProbeError("Created scale-set identity does not match the probe")
        emit("scale_set_created", scale_set_id=self.scale_id, group_id=group_id, label=name)

    def remove_scale(self):
        if self.scale_id is None:
            status, response = self.call(SCALE_PATH + "?" + urllib.parse.urlencode(
                {"runnerGroupId": self.group_id, "name": self.scale_name}))
            expect(status, (200,), "Resolve probe scale-set creation during cleanup")
            rows = response.get("value", [])
            if not rows:
                return
            if self.scale_rejected or len(rows) != 1 or rows[0].get("name") != self.scale_name:
                raise ProbeError("Unconfirmed scale-set identity conflicts; operator inspection is required")
            self.scale_id = int(rows[0]["id"])
            emit("created_scale_set_recovered", scale_set_id=self.scale_id, name=self.scale_name)
        path = SCALE_PATH + "/" + str(self.scale_id)
        status, _ = self.call(path, "DELETE")
        expect(status, (204, 404), "Delete probe scale set")
        status, _ = self.call(path)
        expect(status, (404,), "Verify probe scale set and its sessions are absent")

    def open(self, owner, cleanup):
        path = SCALE_PATH + f"/{self.scale_id}/sessions"
        cleanup.add("probe message session", self.close, 60)
        emit("session_create_intent", scale_set_id=self.scale_id, owner=owner)
        status, session = self.call(path, "POST", {"ownerName": owner})
        expect(status, (200,), "Create probe message session")
        self.session = session
        trusted_url(session["messageQueueUrl"])
        emit("session_created", scale_set_id=self.scale_id)

    def close(self):
        if self.session is None or not self.session.get("sessionId"):
            emit("session_cleanup_via_scale_deletion", scale_set_id=self.scale_id)
            return
        path = SCALE_PATH + f"/{self.scale_id}/sessions/" + segment(self.session["sessionId"])
        status, _ = self.call(path, "DELETE")
        expect(status, (204, 404), "Delete probe message session")
        emit("session_delete_acknowledged", scale_set_id=self.scale_id)

    def refresh_session(self):
        path = SCALE_PATH + f"/{self.scale_id}/sessions/" + segment(self.session["sessionId"])
        status, session = self.call(path, "PATCH")
        expect(status, (200,), "Refresh probe message session")
        trusted_url(session["messageQueueUrl"])
        self.session = session

    def queue_request(self, method="GET", message_id=None):
        for attempt in range(2):
            parsed = urllib.parse.urlsplit(self.session["messageQueueUrl"])
            query = urllib.parse.parse_qs(parsed.query)
            path = parsed.path
            if method == "DELETE":
                path = path.rstrip("/") + "/" + str(message_id)
            elif self.last_message is not None:
                query["lastMessageId"] = [str(self.last_message)]
            url = urllib.parse.urlunsplit(parsed._replace(
                path=path, query=urllib.parse.urlencode(query, doseq=True)))
            status, result = request(url, "Bearer " + self.session["messageQueueAccessToken"],
                                     method, headers={"Accept": "application/json; api-version=6.0-preview",
                                                      "X-ScaleSetMaxCapacity": "1"})
            if status != 401 or attempt == 1:
                return status, result
            self.refresh_session()

    def poll(self):
        status, response = self.queue_request()
        expect(status, (200, 202), "Poll probe message queue")
        if status == 202:
            return None, []
        if response.get("messageType") != "RunnerScaleSetJobMessages":
            raise ProbeError("The scale-set queue returned an unsupported message type")
        try:
            rows = json.loads(response.get("body") or "[]")
        except (TypeError, ValueError):
            raise ProbeError("The scale-set message body was not a job list; body withheld") from None
        return response["messageId"], rows

    def acknowledge(self, message_id):
        status, _ = self.queue_request("DELETE", message_id)
        expect(status, (204,), "Acknowledge processed probe message")
        self.last_message = message_id

    def acquire(self, request_ids):
        status, result = self.call(SCALE_PATH + f"/{self.scale_id}/acquirejobs", "POST", request_ids,
                                   "Bearer " + self.session["messageQueueAccessToken"])
        expect(status, (200,), "Acquire explicit JobAvailable requests")
        if sorted(result["value"]) != sorted(request_ids):
            raise ProbeError("The service did not acquire every explicit available probe job")

    def register(self, name, cleanup):
        path = SCALE_PATH + f"/{self.scale_id}/generatejitconfig"
        runner_id, rejected = None, False

        def remove():
            nonlocal runner_id
            if runner_id is None:
                rows = self.runners_named(name)
                if not rows:
                    return
                if rejected or len(rows) != 1:
                    raise ProbeError("Unconfirmed runner name conflicts; operator inspection is required")
                runner_id = int(rows[0]["id"])
                emit("created_runner_recovered", runner_id=runner_id, runner_name=name)
            self.remove_runner(runner_id, name)

        cleanup.add("runner registration " + name, remove, 50)
        emit("jit_registration_create_intent", scale_set_id=self.scale_id, runner_name=name)
        status, jit = self.call(path, "POST", {"name": name, "workFolder": "_work"})
        rejected = 400 <= status < 500
        expect(status, (200,), "Generate probe JIT registration")
        runner_id = int(jit["runner"]["id"])
        emit("jit_registration_created", runner_id=runner_id, runner_name=name)
        return runner_id, jit["encodedJITConfig"]

    def runners_named(self, name):
        status, response = self.call(RUNNER_PATH + "?" + urllib.parse.urlencode({"agentName": name}))
        expect(status, (200,), "Resolve probe runner name")
        return [row for row in response.get("value", []) if row.get("name") == name]

    def remove_runner(self, runner_id, name):
        path = RUNNER_PATH + "/" + str(runner_id)
        for attempt in range(30):
            status, _ = self.call(path, "DELETE")
            remaining = self.runners_named(name)
            if not remaining:
                emit("runner_registration_absence_verified", runner_id=runner_id, runner_name=name)
                return
            if len(remaining) != 1 or int(remaining[0]["id"]) != runner_id:
                raise ProbeError("Probe runner identity changed during cleanup")
            expect(status, (204, 400, 404, 409, 422), "Delete probe runner registration")
            time.sleep(2)
        raise ProbeError("The probe runner registration remains after cleanup")
