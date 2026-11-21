"""Keep overflow disabled until every durable start of the smoke has ended."""

import subprocess
import time
from urllib.parse import quote
from uuid import UUID


CLEANUP_SECONDS = 90
ERRORS = (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError)


def workflows_finished(live, cancelled):
    workflow = live.config["workflow_id"]
    route = "/actions/workflows/" + quote(str(workflow), safe="")
    runs = live.github("GET", route + "/runs?event=workflow_dispatch&per_page=100")["workflow_runs"]
    titles = {"ciproxy-smoke-" + marker for marker in live.markers}
    found = [run for run in runs if run["display_title"] in titles]
    finished = len(found) == len(live.markers)
    for run in found:
        route = "/actions/runs/" + str(run["id"])
        if run["status"] != "completed" and run["id"] not in cancelled:
            live.github("POST", route + "/cancel", {})
            cancelled.add(run["id"])
        if live.github("GET", route)["status"] != "completed":
            finished = False
    return finished


def unfinished(deliveries, verified):
    return [row for row in deliveries.values()
            if row["ends_at"] is None
            or (row["sandbox"] is not None and row["sandbox"]["id"] not in verified)]


def observe(live, deliveries):
    for row in live.created():
        deliveries[row["delivery"]] = row
    return live.new_intents()


def poll(live, deadline):
    disabled = False
    workflows = not live.markers
    read = not live.markers
    intents = []
    deliveries = {}
    verified = set()
    cancelled = set()
    while time.monotonic() < deadline:
        if not disabled:
            try:
                live.api("PUT", "/overflow", {"enabled": False,
                    "wait_seconds": live.original["wait_seconds"],
                    "spend_limit_cents": live.original["spend_limit_cents"]})
                disabled = True
            except ERRORS:
                pass
        if live.markers:
            read = False
            try:
                intents = observe(live, deliveries)
                read = True
            except ERRORS:
                pass
            previously_finished = workflows
            if not workflows:
                try:
                    workflows = workflows_finished(live, cancelled)
                except ERRORS:
                    pass
            if workflows and not previously_finished:
                read = False
                try:
                    intents = observe(live, deliveries)
                    read = True
                except ERRORS:
                    pass
            if workflows:
                for row in deliveries.values():
                    sandbox = row["sandbox"]
                    if sandbox is not None and sandbox["id"] not in verified:
                        try:
                            live.destroy(sandbox["id"])
                            verified.add(sandbox["id"])
                        except ERRORS:
                            pass
        pending = unfinished(deliveries, verified)
        if disabled and workflows and read and not intents and not pending:
            try:
                live.api("PUT", "/overflow", {key: live.original[key] for key in
                    ("enabled", "wait_seconds", "spend_limit_cents")})
                return []
            except ERRORS:
                return ["settings restoration"]
        remaining = deadline - time.monotonic()
        if remaining > 0:
            time.sleep(min(1, remaining))
    failures = []
    if not disabled:
        failures.append("disable new Boat dispatch")
    if not workflows:
        failures.append("workflow cancellation or lost-dispatch recovery")
    if not read:
        failures.append("durable start state could not be confirmed")
    pending = unfinished(deliveries, verified)
    if pending:
        ids = sorted(str(UUID(row["delivery"])) for row in pending)
        failures.append("unresolved Boat deliveries: " + ", ".join(ids))
    if intents:
        ids = sorted(str(UUID(row["runner"])) for row in intents)
        failures.append("unresolved Boat runner intents: " + ", ".join(ids))
    failures.append("cleanup deadline reached; settings restoration remains held")
    return failures


def restore(live):
    if not live.policy_attempted and not live.markers:
        return []
    deadline = time.monotonic() + CLEANUP_SECONDS
    live.cleanup_deadline = deadline
    try:
        return poll(live, deadline)
    finally:
        live.cleanup_deadline = None
