"""Live cache hit, isolation, and service-failure checks."""

import os
from pathlib import Path

from own_smoke import Failure
from own_scenarios import Skip


def cache(smoke):
    path = os.environ.get("CIPROXY_CACHE_ADMIN_TOKEN_FILE")
    if not path:
        raise Skip("Set CIPROXY_CACHE_ADMIN_TOKEN_FILE to the host cache administration token file")
    token = Path(path).read_text().strip()
    first = smoke.dispatch("cache", cache_key=smoke.tag)
    smoke.local_job(first)
    smoke.finish(first)
    second = smoke.dispatch("cache", cache_key=smoke.tag)
    smoke.local_job(second)
    smoke.finish(second)
    if first["vm"] == second["vm"]:
        raise Failure("The cache check reused a job VM")
    report = smoke.request("http://127.0.0.1:8229", "GET", "/report", {"Authorization": "Bearer " + token})
    usage = next((job["usage"] for job in report["jobs"] if job["job_id"] == second["job"]), None)
    if not usage or usage.get("handling") != "machine_handled" or int(usage.get("local_entries", 0)) < 1:
        raise Failure("The fresh second VM had no machine-cache restore")
    smoke.record("cache", {"job": second["job"], "local_entries": usage["local_entries"], "restored_bytes": usage["restored_bytes"]})



def cache_isolation(smoke):
    repository=smoke.config.get("cache_isolation_repository")
    writer_ref=smoke.config.get("cache_writer_ref")
    reader_ref=smoke.config.get("cache_reader_ref")
    if not all([repository,writer_ref,reader_ref]):
        raise Skip("Configure cache_isolation_repository, cache_writer_ref, and cache_reader_ref")
    key=smoke.tag+"-private"
    writer=smoke.dispatch("cache",cache_key=key,ref=writer_ref,cache_expectation="miss")
    smoke.local_job(writer)
    smoke.finish(writer)
    other=smoke.dispatch("cache",repository=repository,cache_key=key,cache_expectation="miss")
    smoke.local_job(other)
    smoke.finish(other)
    reader=smoke.dispatch("cache",cache_key=key,ref=reader_ref,cache_expectation="miss")
    smoke.local_job(reader)
    smoke.finish(reader)
    smoke.record("cache_isolation",{"private_writer_run":writer["id"],"other_repository_run":other["id"],"other_ref_run":reader["id"],"private_scope_not_restored":True})


def cache_fallback(smoke):
    key=smoke.tag+"-fallback"
    first=smoke.dispatch("cache",cache_key=key,cache_expectation="miss")
    smoke.local_job(first)
    smoke.finish(first)
    try:
        smoke.command("systemctl","mask","--runtime","--now","ciproxy-cache.service")
        second=smoke.dispatch("cache",cache_key=key,cache_expectation="hit")
        smoke.local_job(second)
        smoke.finish(second)
    finally:
        smoke.command("systemctl","unmask","--runtime","ciproxy-cache.service")
        smoke.command("systemctl","start","ciproxy-cache.service")
    smoke.record("cache_fallback",{"run":second["id"],"service_unavailable":True,"github_restore_succeeded":True})
