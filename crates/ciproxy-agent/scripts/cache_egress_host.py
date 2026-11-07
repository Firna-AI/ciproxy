"""Private-file access and probes against the installed machine cache."""

import json
import os
from pathlib import Path
import pwd
import socket
import struct
import subprocess
import sys
import urllib.error
import urllib.request
import uuid


class Failure(Exception):
    """A failed proof with a safe, fixed diagnostic."""


def require(condition, message):
    if not condition:
        raise Failure(message)


def command(*args, check=True, input=None):
    result = subprocess.run(args, input=input, capture_output=True, text=True)
    require(not check or result.returncode == 0, args[0] + " failed")
    return result


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args):
        return None


def request(url, method="GET", headers=None, body=None, timeout=20):
    data = None if body is None else json.dumps(body).encode()
    headers = dict(headers or {})
    if data is not None:
        headers["Content-Type"] = "application/json"
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
    try:
        with opener.open(urllib.request.Request(url, data, headers, method=method),
                         timeout=timeout) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()


class InstalledCache:
    def __init__(self):
        self.account = pwd.getpwnam("ciproxy-cache")
        require(self.account.pw_uid > 0, "The cache account is root")
        self.headers = {"Authorization": "Bearer " + Path(
            os.environ["CIPROXY_CACHE_ADMIN_TOKEN_FILE"]).read_text().strip()}

    def admin(self, path, method="GET", body=None):
        status, raw = request("http://127.0.0.1:8229" + path, method, self.headers, body)
        require(200 <= status < 300, "Cache administration did not succeed")
        return json.loads(raw) if raw else None

    def inspect(self):
        pid = int(command("systemctl", "show", "-p", "MainPID", "--value",
                          "ciproxy-cache.service").stdout)
        require(pid > 0, "The installed cache service is not running")
        status = dict(line.split(":", 1) for line in Path(f"/proc/{pid}/status")
                      .read_text().splitlines() if ":" in line)
        require(set(status["Uid"].split()) == {str(self.account.pw_uid)},
                "The installed cache process has the wrong UID")
        require(int(status["CapEff"], 16) == 0, "The cache retains kernel capabilities")
        require(status["NoNewPrivs"].strip() == "1", "The cache can gain privileges")
        for path, mode in [("/var/lib/ciproxy", 0o700),
                           ("/var/lib/ciproxy/area/cache", 0o700),
                           (os.environ["CIPROXY_CACHE_ADMIN_TOKEN_FILE"], 0o600)]:
            metadata = Path(path).stat()
            require(metadata.st_uid == 0 and metadata.st_mode & 0o777 == mode,
                    "Root cache control permissions changed")
        data = Path("/var/lib/ciproxy/area/cache/data").stat()
        require(data.st_uid == self.account.pw_uid and data.st_mode & 0o777 == 0o700,
                "Cache data has the wrong owner or permissions")
        mounted = Path(f"/proc/{pid}/root/run/ciproxy-cache/data").stat()
        require((mounted.st_dev, mounted.st_ino) == (data.st_dev, data.st_ino),
                "The cache does not use its installed data bind mount")
        for path, mode in [("/run/ciproxy-cache-auth", 0o710),
                           ("/run/ciproxy-cache-auth/admin-token", 0o640)]:
            metadata = Path(path).stat()
            require(metadata.st_uid == 0 and metadata.st_gid == self.account.pw_gid
                    and metadata.st_mode & 0o777 == mode,
                    "The transient cache credential has unsafe ownership or permissions")
        self.admin("/report")
        command("nft", "list", "chain", "inet", "ciproxy_cache", "output")
        denied = """from pathlib import Path
import sys
try:
    Path('/var/lib/ciproxy/state.json').open('rb').close()
except PermissionError:
    sys.exit(0)
sys.exit(1)
"""
        command("setpriv", "--reuid=" + str(self.account.pw_uid),
                "--regid=" + str(self.account.pw_gid), "--clear-groups",
                sys.executable, "-c", denied)
        return pid

    def drops(self, chain="service"):
        rules = json.loads(command("nft", "-j", "list", "chain", "inet",
                                   "ciproxy_cache", chain).stdout)["nftables"]
        return sum(expression["counter"]["packets"] for item in rules
                   for expression in item.get("rule", {}).get("expr", [])
                   if "counter" in expression)

    def forward(self, upstream, expected):
        job = str(uuid.uuid4())
        registration = {
            "job_id": job, "facts": {
                "organization_id": "0", "repository": {
                    "id": "0", "full_name": "public/firewall-probe"},
                "assignment": {"run_id": "0", "job_id": job, "runner_id": "0"},
                "event": "workflow_dispatch", "head_repository": None,
                "default_ref": "refs/heads/main", "sticky_docker": False},
            "permissions": {"read": True, "write": False}, "github_token": "",
            "cache_url": upstream, "results_url": upstream,
            "proxy_url": "http://127.0.0.1:7887"}
        session = self.admin("/sessions", "POST", registration)["token"]
        try:
            status, _ = request("http://127.0.0.1:7887/", headers={
                "x-ciproxy-session": session})
            require(status == expected, "The installed cache returned an unexpected probe status")
        finally:
            self.admin("/sessions/" + session, "DELETE")

    def blocked(self, upstream):
        before = self.drops()
        self.forward(upstream, 502)
        require(self.drops() > before, "The installed output firewall did not drop the probe")

    def dns(self):
        result = command("setpriv", "--reuid=" + str(self.account.pw_uid),
                         "--regid=" + str(self.account.pw_gid), "--clear-groups",
                         sys.executable, "-", input=Path(__file__).read_text())
        require(int(result.stdout) > 0, "No configured resolver answered the cache UID")
        return int(result.stdout)


def dns_probe():
    count = 0
    query = b"\x43\x50\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00" + (
        b"\x07example\x03com\x00\x00\x01\x00\x01")
    for line in Path("/etc/resolv.conf").read_text().splitlines():
        parts = line.split()
        if len(parts) < 2 or parts[0] != "nameserver":
            continue
        resolver = parts[1]
        family = socket.AF_INET6 if ":" in resolver else socket.AF_INET
        for kind in [socket.SOCK_DGRAM, socket.SOCK_STREAM]:
            address = socket.getaddrinfo(resolver, 53, family, kind,
                                         flags=socket.AI_NUMERICHOST)[0][4]
            with socket.socket(family, kind) as connection:
                connection.settimeout(10)
                connection.connect(address)
                if kind == socket.SOCK_STREAM:
                    connection.sendall(struct.pack("!H", len(query)) + query)
                    length = bytearray()
                    while len(length) < 2:
                        chunk = connection.recv(2 - len(length))
                        require(chunk, "A resolver closed its TCP reply")
                        length.extend(chunk)
                    remaining = struct.unpack("!H", length)[0]
                    reply = bytearray()
                    while len(reply) < remaining:
                        chunk = connection.recv(remaining - len(reply))
                        require(chunk, "A resolver truncated its TCP reply")
                        reply.extend(chunk)
                else:
                    connection.send(query)
                    reply = connection.recv(4096)
                require(reply[:2] == query[:2] and len(reply) >= 12,
                        "A configured resolver returned an invalid DNS reply")
                count += 1
    print(count)


if __name__ == "__main__":
    try:
        dns_probe()
    except (Failure, OSError, ValueError):
        print("FAIL: configured resolver DNS probe failed", file=sys.stderr)
        sys.exit(1)
