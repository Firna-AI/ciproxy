"""Live scenario checks. These require a disposable registered KVM machine."""

import json
import os
from pathlib import Path
import time

from own_smoke import Failure


class Skip(Exception):
    """A scenario lacks a required live resource."""


def warm(smoke):
    before = smoke.state()["runners"]
    ready = {vm["report"]["runner_id"] for vm in smoke.wait(smoke.warm, "warm readiness")}
    original = {vm["local_id"] for vm in before}
    run = smoke.dispatch("warm")
    vm = smoke.local_job(run)
    if vm["report"]["runner_id"] not in ready:
        raise Failure("The job did not use a runner that was already warm")
    smoke.finish(run)
    smoke.wait(lambda: any(vm["local_id"] not in original for vm in smoke.warm()), "fresh warm replacement")


def slots(smoke):
    count = smoke.config.get("parallel_jobs", 2)
    view = smoke.api("GET", smoke.machine_path())
    if int(view["concurrent_default_jobs"]) < count:
        raise Skip("This machine cannot run the configured parallel default jobs")
    runs = [smoke.dispatch() for _ in range(count)]
    machines = [smoke.local_job(run) for run in runs]
    active = {vm["local_id"] for vm in smoke.state()["runners"] if vm["report"]["state"]["state"] == "running"}
    if not all(vm["local_id"] in active for vm in machines):
        raise Failure("The jobs did not overlap on this machine")
    held = [slot for vm in machines for slot in vm["report"]["slot_indices"]]
    if not held or len(set(held)) != len(held):
        raise Failure("Concurrent VMs did not hold distinct reported slots")
    for run in runs:
        smoke.finish(run)


def drain(smoke):
    run = smoke.dispatch()
    vm = smoke.local_job(run)
    try:
        smoke.action("drain")
        smoke.wait(lambda: smoke.state()["mode"] == "Draining" and not smoke.warm(), "drain removes idle runners")
        if not any(value["local_id"] == vm["local_id"] for value in smoke.state()["runners"]):
            raise Failure("The active job ended before drain preservation was observed")
        smoke.finish(run)
    finally:
        smoke.action("resume")
    smoke.wait(smoke.warm, "warm runners resume")


def offline(smoke):
    run = smoke.dispatch()
    vm = smoke.local_job(run)
    before = smoke.pid(vm)
    try:
        smoke.command("systemctl", "stop", "ciproxy-agent.service")
        smoke.wait(lambda: smoke.api("GET", smoke.machine_path())["offline"], "machine becomes offline")
        if smoke.pid(vm) != before or before == "0":
            raise Failure("Stopping the agent changed the active VM process")
    finally:
        smoke.command("systemctl", "start", "ciproxy-agent.service")
    smoke.wait(lambda: not smoke.api("GET", smoke.machine_path())["offline"], "machine reconnects")
    smoke.finish(run)


def restart(smoke):
    run = smoke.dispatch("restart")
    vm = smoke.local_job(run)
    before = smoke.pid(vm)
    invocation = smoke.command("systemctl", "show", "--property=InvocationID", "--value", "ciproxy-agent.service", capture=True)
    smoke.command("systemctl", "restart", "ciproxy-agent.service")
    after = smoke.command("systemctl", "show", "--property=InvocationID", "--value", "ciproxy-agent.service", capture=True)
    if after == invocation or smoke.pid(vm) != before or before == "0":
        raise Failure("Restart did not preserve the VM in a new agent invocation")
    def replayed():
        rows = smoke.command("journalctl", "--quiet", "--no-pager", "--output=json", "_SYSTEMD_INVOCATION_ID=" + after, capture=True)
        for row in rows.splitlines():
            try:
                fields = json.loads(json.loads(row)["MESSAGE"])["fields"]
            except (ValueError, KeyError, TypeError):
                continue
            if fields.get("event") == "job_started_observed" and fields.get("vm_id") == vm["local_id"]:
                return True
        return False
    smoke.wait(replayed, "new agent accepts the guest's replayed assignment")
    smoke.record("restart", {"vm": vm["local_id"], "same_pid": True, "new_invocation": after})
    smoke.finish(run)



def disallowed(smoke):
    repository = smoke.config.get("disallowed_repository")
    if not repository:
        raise Skip("Configure a private disallowed_repository with the same workflow fixture")
    blocked = smoke.dispatch("docker", repository=repository)
    control = smoke.dispatch("docker")
    smoke.local_job(control)
    smoke.finish(control)
    smoke.wait(smoke.warm, "allowed warm capacity is available")
    observed_queued=False
    deadline = time.monotonic() + smoke.config.get("queued_observation_seconds", 60)
    while time.monotonic() < deadline:
        jobs = smoke.jobs(blocked)
        observed_queued=observed_queued or bool(jobs)
        if any(job["status"] != "queued" or job.get("runner_id") for job in jobs):
            raise Failure("A disallowed repository received a runner")
        time.sleep(1)
    if not observed_queued:
        raise Failure("The disallowed workflow never exposed a queued job")
    smoke.record("disallowed", {"id": blocked["id"], "still_queued": True})


def remove(smoke):
    if not smoke.config.get("item_79_policy_verified",False):
        raise Skip("Item 79 removal policy must be selected and verified before the removal scenario")
    run = smoke.dispatch()
    vm = smoke.local_job(run)
    before = smoke.pid(vm)
    smoke.action("remove")
    smoke.wait(lambda: smoke.api("GET", smoke.machine_path(), absent=True) is None, "removed machine is hidden")
    if smoke.pid(vm) != before or before == "0":
        raise Failure("Removal changed an already-observed running VM process")
    smoke.finish(run)
    smoke.wait(lambda: smoke.state()["mode"] == "Removed" and not smoke.state()["runners"], "removed agent finishes and releases its VMs")
