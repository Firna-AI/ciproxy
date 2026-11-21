"""Live trusted writer, disposable-copy, and no-fit Docker checks."""

import json
from pathlib import Path

from own_smoke import Failure
from own_scenarios import Skip


def prepared(smoke, vm):
    def observed():
        rows = smoke.command("journalctl", "--quiet", "--no-pager", "--output=json", "-u", "ciproxy-agent.service", capture=True)
        for row in rows.splitlines():
            try:
                fields = json.loads(json.loads(row)["MESSAGE"])["fields"]
            except (ValueError, KeyError, TypeError):
                continue
            if fields.get("event") == "job_prepared_observed" and fields.get("vm_id") == vm["local_id"]:
                return True
        return False
    smoke.wait(observed, "host observes actual guest readiness")


def lease(smoke, vm, writer):
    prepared(smoke, vm)
    path = Path("/var/lib/ciproxy/area/docker-leases") / vm["local_id"]
    if not path.is_file():
        raise Failure("The job ran without the required sticky Docker attachment")
    value = json.loads(path.read_text())
    if value["writer"] != writer or int(value["bytes"]) <= 0:
        raise Failure("The host Docker lease has the wrong trusted access mode")
    disk = Path("/var/lib/ciproxy/area/jailer/firecracker") / vm["local_id"] / "root/docker-job.ext4"
    if disk.stat().st_size != int(value["bytes"]):
        raise Failure("The attachment length differs from its durable reservation")
    smoke.record("docker_attachment", {"vm": vm["local_id"], "repository": value["repository"],
                                       "writer": writer, "reserved_bytes": value["bytes"]})
    return value


def sticky_docker(smoke):
    if not smoke.config.get("sticky_tier_ready") or not smoke.config.get("sticky_copy_ref"):
        raise Skip("Prepare Team/Business, enough spare disk, and sticky_copy_ref with the fixture")
    marker = smoke.tag + "-sticky"
    first = smoke.dispatch("sticky-write", cache_key=marker)
    first_vm = smoke.local_job(first)
    original = lease(smoke, first_vm, True)
    smoke.finish(first)
    reader = smoke.dispatch("sticky-read", cache_key=marker)
    reader_vm = smoke.local_job(reader)
    retained = lease(smoke, reader_vm, True)
    if retained["repository"] != original["repository"] or reader_vm["local_id"] == first_vm["local_id"]:
        raise Failure("The persistence check did not use one repository disk across fresh VMs")
    smoke.finish(reader)
    copied = smoke.dispatch("sticky-copy", cache_key=marker, ref=smoke.config["sticky_copy_ref"])
    copied_vm = smoke.local_job(copied)
    lease(smoke, copied_vm, False)
    smoke.finish(copied)
    final = smoke.dispatch("sticky-read", cache_key=marker)
    final_vm = smoke.local_job(final)
    lease(smoke, final_vm, True)
    smoke.finish(final)
    smoke.record("docker_persistence", {"writer_run": first["id"], "reader_run": reader["id"],
                                        "copy_run": copied["id"], "final_run": final["id"],
                                        "untrusted_copy_changes_discarded": True})


def docker_no_fit(smoke):
    if not smoke.config.get("docker_no_fit_ready"):
        raise Skip("Prepare a disk-limited machine with zero spare disk for this separate scenario")
    run = smoke.dispatch("docker", runner_label=smoke.config.get("runner_label", "ciproxy"))
    vm = smoke.local_job(run)
    prepared(smoke, vm)
    budget = smoke.state()["docker_budget"]
    if budget is not None and int(budget["spare_bytes"]) != 0:
        raise Failure("The no-fit check needs a host policy with zero spare Docker bytes")
    if (Path("/var/lib/ciproxy/area/docker-leases") / vm["local_id"]).exists():
        raise Failure("The no-fit job unexpectedly received a Docker reservation")
    smoke.finish(run)
    smoke.record("docker_no_fit", {"run": run["id"], "root_docker_succeeded": True})
