"""Raw queue observation with capacity, cancellation, and no runner registration."""

import json
import time
import urllib.parse

from probe_common import ProbeError, emit, expect, request
from probe_scales import ScaleService

MESSAGE_FIELDS = ("jobWorkflowRef", "eventName", "jobDisplayName", "workflowRunId")
RUN_FIELDS = ("id", "event", "head_branch", "head_sha", "path", "run_attempt", "status", "conclusion")


def run_facts(row):
    facts = {key: row[key] for key in RUN_FIELDS if key in row}
    for key in ("repository", "head_repository"):
        source = row.get(key)
        facts[key] = None if source is None else {name: source[name]
            for name in ("id", "full_name", "private") if name in source}
    facts["pull_requests"] = [{"number": item.get("number"),
        **{side: {key: (item.get(side) or {}).get(key) for key in ("ref", "sha")}
           for side in ("head", "base")}} for item in row.get("pull_requests") or []]
    facts["referenced_workflows"] = [{key: item[key] for key in ("path", "ref", "sha") if key in item}
                                    for item in row.get("referenced_workflows") or []]
    return facts


class RunnerFreeScale(ScaleService):
    def register(self, *args, **kwargs):
        raise ProbeError("This probe cannot create a runner or JIT configuration")

    def acknowledge(self, message_id):
        for attempt in range(2):
            parsed = urllib.parse.urlsplit(self.session["messageQueueUrl"])
            url = urllib.parse.urlunsplit(parsed._replace(path=parsed.path.rstrip("/") + "/" + str(message_id)))
            status, _ = request(url, "Bearer " + self.session["messageQueueAccessToken"], "DELETE")
            if status != 401 or attempt == 1:
                break
            self.refresh_session()
        expect(status, (204,), "Acknowledge observed runner-free message")
        self.last_message = message_id

    def packet(self, capacity):
        for attempt in range(2):
            parsed = urllib.parse.urlsplit(self.session["messageQueueUrl"])
            query = urllib.parse.parse_qs(parsed.query)
            if self.last_message is not None:
                query["lastMessageId"] = [str(self.last_message)]
            url = urllib.parse.urlunsplit(parsed._replace(query=urllib.parse.urlencode(query, doseq=True)))
            status, response = request(url, "Bearer " + self.session["messageQueueAccessToken"], headers={
                "Accept": "application/json; api-version=6.0-preview", "X-ScaleSetMaxCapacity": str(capacity)}, timeout=65)
            if status != 401 or attempt == 1:
                break
            self.refresh_session()
        expect(status, (200, 202), "Poll runner-free scale-set queue")
        if status == 202:
            return None, []
        if not isinstance(response, dict) or response.get("messageType") != "RunnerScaleSetJobMessages":
            raise ProbeError("Unexpected queue envelope; response withheld and unacknowledged")
        message_id = response.get("messageId")
        if type(message_id) is not int or not isinstance(response.get("body"), str):
            raise ProbeError("Malformed queue envelope; response withheld and unacknowledged")
        try:
            rows = json.loads(response["body"])
        except ValueError:
            raise ProbeError("Malformed queue body; response withheld and unacknowledged") from None
        if not isinstance(rows, list) or any(not isinstance(row, dict) for row in rows):
            raise ProbeError("Malformed queue entries; response withheld and unacknowledged")
        return message_id, rows


class Observer:
    def __init__(self, app, service, resources, capacity):
        self.app, self.service, self.resources, self.capacity = app, service, resources, capacity
        self.cases, self.seen, self.counts = {}, set(), {}

    def add_cases(self, cases):
        for item in cases:
            key = str(item["run_id"])
            self.cases[key] = item
            self.counts[key] = 0

    def consume(self, rows):
        for row in rows:
            kind = row.get("messageType")
            if kind == "JobAvailable":
                if str(row.get("workflowRunId")) not in self.cases:
                    raise ProbeError("An available job is outside the owned fixture runs")
                request_id = row.get("runnerRequestId")
                if type(request_id) is not int or request_id <= 0:
                    raise ProbeError("JobAvailable has no usable request identity; batch unacknowledged")
                self.service.acquire([request_id])
                continue
            if kind == "JobStarted":
                raise ProbeError("A job started during a runner-free probe; batch unacknowledged")
            if kind != "JobAssigned":
                continue
            if (any(not isinstance(row.get(key), str) or not row[key] for key in MESSAGE_FIELDS[:3])
                    or not str(row.get("workflowRunId", "")).isdigit()
                    or not isinstance(row.get("jobId"), str) or not row["jobId"]):
                raise ProbeError("JobAssigned lacks required metadata; batch unacknowledged")
            run_id = str(row["workflowRunId"])
            if run_id not in self.cases:
                raise ProbeError("The fixed label received an assignment outside the owned fixture runs")
            item = self.cases[run_id]
            identity = (run_id, row["jobId"])
            if identity in self.seen:
                continue
            status, api = self.app.call(self.resources.prefix + "/actions/runs/" + run_id)
            expect(status, (200,), "Read assigned run facts with App Actions read")
            if str(api.get("id")) != run_id or api.get("event") != item["event"]:
                raise ProbeError("The assigned run API identity differs from the owned fixture")
            self.seen.add(identity)
            self.counts[run_id] += 1
            emit("workflow_ref_assignment", case=item["name"],
                 message={key: row[key] for key in MESSAGE_FIELDS}, run=run_facts(api))

    def collect(self, cases, deadline):
        self.add_cases(cases)
        while time.monotonic() < deadline:
            if all(self.counts[str(item["run_id"])] >= item["expected"] for item in cases):
                return
            message_id, rows = self.service.packet(self.capacity)
            self.consume(rows)
            if message_id is not None:
                self.service.acknowledge(message_id)
        raise ProbeError("Required JobAssigned messages were not observed before the time limit")

    def cancel_and_drain(self, cases, deadline):
        for item in cases:
            self.resources.cancel(item["run_id"])
        while time.monotonic() < deadline:
            message_id, rows = self.service.packet(0)
            self.consume(rows)
            if message_id is None:
                emit("owned_runs_cancelled_queue_empty", runs=[item["run_id"] for item in cases], capacity=0)
                return
            self.service.acknowledge(message_id)
        raise ProbeError("The queue did not become empty after owned run cancellation")
