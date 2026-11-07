"""Installed service starts, automatic restarts, and firewall-failure fallback."""

from pathlib import Path
import socket
import time
import uuid

from cache_egress_host import command, require
from cache_egress_network import blocked_target, host_listener


POLICY = Path("/etc/ciproxy-cache.nft")
INVALID_POLICY = b"invalid ciproxy firewall smoke input\n"


def pid():
    return int(command("systemctl", "show", "-p", "MainPID", "--value",
                       "ciproxy-cache.service").stdout)


def wait_started(cache, previous=0):
    deadline = time.monotonic() + 60
    while time.monotonic() < deadline:
        current = pid()
        if current > 0 and current != previous:
            try:
                cache.inspect()
                return
            except OSError:
                pass
        time.sleep(1)
    require(False, "The protected cache service did not start")


def no_listener():
    require(pid() == 0, "A failed firewall load left a cache process running")
    require(not Path("/run/ciproxy-cache-auth/admin-token").exists(),
            "A failed firewall load retained the transient administration credential")
    try:
        with socket.create_connection(("127.0.0.1", 7887), timeout=5):
            reached = True
    except OSError:
        reached = False
    require(not reached, "A failed firewall load left a cache listener open")


def lifecycle(cache):
    original = POLICY.read_bytes()
    agent_active = command("systemctl", "is-active", "--quiet",
                           "ciproxy-agent.service", check=False).returncode == 0
    marker = "firewall-probe-" + uuid.uuid4().hex
    parent_marker = Path("/var/lib/ciproxy/area/cache") / marker
    child_marker = parent_marker.parent / "data" / marker
    try:
        if agent_active:
            command("systemctl", "stop", "ciproxy-agent.service")
        command("systemctl", "stop", "ciproxy-cache.service")
        parent_marker.write_bytes(b"public cache migration probe\n")
        command("nft", "delete", "table", "inet", "ciproxy_cache")
        command("systemctl", "start", "ciproxy-cache.service")
        wait_started(cache)
        require(child_marker.read_bytes() == b"public cache migration probe\n",
                "A protected start lost legacy cache data")
        require(child_marker.stat().st_uid == cache.account.pw_uid,
                "A protected start did not transfer legacy cache ownership")
        with host_listener("4") as port:
            blocked_target(cache, f"http://127.0.0.1:{port}/")
            previous = pid()
            command("systemctl", "kill", "--kill-who=main", "--signal=KILL",
                    "ciproxy-cache.service")
            wait_started(cache, previous)
            blocked_target(cache, f"http://127.0.0.1:{port}/")
        command("systemctl", "stop", "ciproxy-cache.service")
        POLICY.write_bytes(INVALID_POLICY)
        command("nft", "delete", "table", "inet", "ciproxy_cache")
        result = command("systemctl", "start", "ciproxy-cache.service", check=False)
        require(result.returncode != 0, "A cache start ignored the invalid firewall policy")
        no_listener()
    finally:
        POLICY.write_bytes(original)
        command("systemctl", "reset-failed", "ciproxy-cache.service")
        command("systemctl", "start", "ciproxy-cache.service")
        parent_marker.unlink(missing_ok=True)
        child_marker.unlink(missing_ok=True)
        if agent_active:
            command("systemctl", "start", "ciproxy-agent.service")
    wait_started(cache)


def firewall_fallback(smoke, cache):
    key = smoke.tag + "-firewall-fallback"
    first = smoke.dispatch("cache", cache_key=key, cache_expectation="miss")
    smoke.local_job(first)
    smoke.finish(first)
    original = POLICY.read_bytes()
    try:
        command("systemctl", "stop", "ciproxy-cache.service")
        POLICY.write_bytes(INVALID_POLICY)
        result = command("systemctl", "start", "ciproxy-cache.service", check=False)
        require(result.returncode != 0, "The invalid output policy did not prevent cache startup")
        no_listener()
        second = smoke.dispatch("cache", cache_key=key, cache_expectation="hit")
        smoke.local_job(second)
        smoke.finish(second)
        no_listener()
        require(POLICY.read_bytes() == INVALID_POLICY,
                "Host repair removed the firewall-failure condition during the job")
        smoke.record("firewall_fallback", {"run": second["id"], "github_restore_succeeded": True})
    finally:
        POLICY.write_bytes(original)
        command("systemctl", "reset-failed", "ciproxy-cache.service")
        command("systemctl", "start", "ciproxy-cache.service")
    cache.inspect()
