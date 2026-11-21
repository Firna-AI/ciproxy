"""Filesystem, service, clock, and HTTP boundaries for the installed cache."""

from contextlib import contextmanager
import hashlib
import io
import json
from pathlib import PurePosixPath
from types import SimpleNamespace
from urllib.error import HTTPError
from urllib.parse import urlsplit
from unittest.mock import patch

import cache_egress_host as host
import cache_egress_lifecycle as lifecycle


class File:
    def __init__(self, state, path):
        self.state, self.path = state, PurePosixPath(path)

    @property
    def parent(self):
        return File(self.state, self.path.parent)

    def __truediv__(self, child):
        return File(self.state, self.path / child)

    def read_bytes(self):
        return self.state.contents(str(self.path))

    def read_text(self):
        return self.read_bytes().decode()

    def write_bytes(self, value):
        self.state.files[str(self.path)] = value

    def stat(self):
        name = str(self.path)
        owner, group, mode = 0, 0, 0o700
        if name.endswith("cache-admin-token"):
            mode = 0o600
        elif name == "/run/ciproxy-cache-auth":
            group, mode = 222, 0o710
        elif name == "/run/ciproxy-cache-auth/admin-token":
            group, mode = 222, 0o640
        elif "/data" in name:
            owner = 222
        return SimpleNamespace(st_uid=owner, st_gid=group, st_mode=mode,
                               st_dev=1, st_ino=20)

    def exists(self):
        return str(self.path) in self.state.files

    def unlink(self, missing_ok=False):
        self.state.files.pop(str(self.path), None)


class CacheHost:
    token_file = "/private/cache-admin-token"

    def __init__(self, replay="ready", authorized=True, denies_anonymous=True):
        self.files = {self.token_file: b"public-admin-probe",
                      "/etc/ciproxy-cache.nft": b"public valid policy\n",
                      "/run/ciproxy-cache-auth/admin-token": b""}
        self.unit = b"[Service]\nExecStart=/public/cache-sidecar\n"
        self.digest = hashlib.sha256(self.unit).hexdigest().encode()
        self.account = SimpleNamespace(pw_uid=222, pw_gid=222)
        self.current_pid = 500
        self.agent_active = True
        self.resumed = False
        self.budget = None
        self.replay = replay
        self.authorized = authorized
        self.denies_anonymous = denies_anonymous
        self.now = 0
        self.dropped = 0
        self.requests = []
        self.commands = []
        self.reports = 0

    def path(self, value):
        return File(self, value)

    def contents(self, name):
        if name == "/etc/systemd/system/ciproxy-cache.service":
            return self.unit + b"Environment=CIPROXY_CACHE_UNIT_SHA256=" + self.digest + b"\n"
        if name == f"/proc/{self.current_pid}/environ":
            return b"CIPROXY_CACHE_UNIT_SHA256=" + self.digest + b"\0"
        if name == f"/proc/{self.current_pid}/status":
            return b"Uid:\t222 222 222 222\nCapEff:\t0\nNoNewPrivs:\t1\n"
        return self.files[name]

    def run(self, args, **kwargs):
        self.commands.append(args)
        status, output = 0, ""
        if args[:3] == ("systemctl", "show", "-p"):
            output = str(self.current_pid)
        elif args[:2] == ("systemctl", "is-active"):
            status = 0 if self.agent_active else 3
        elif args[0] == "systemctl" and args[-1] == "ciproxy-agent.service":
            self.agent_active = args[1] == "start"
            self.resumed = self.agent_active
        elif args[0] == "systemctl" and args[-1] == "ciproxy-cache.service":
            if args[1] == "stop":
                self.current_pid = 0
            elif args[1] in {"start", "kill"}:
                self.budget = None
                if self.files["/etc/ciproxy-cache.nft"] == lifecycle.INVALID_POLICY:
                    self.current_pid = 0
                    self.files.pop("/run/ciproxy-cache-auth/admin-token", None)
                    status = 1
                else:
                    self.current_pid += 500
                    self.files["/run/ciproxy-cache-auth/admin-token"] = b""
                    parent = "/var/lib/ciproxy/area/cache/"
                    for name in list(self.files):
                        if name.startswith(parent + "firewall-probe-"):
                            self.files[parent + "data/" + name[len(parent):]] = self.files.pop(name)
        elif args[:3] == ("nft", "-j", "list"):
            output = json.dumps({"nftables": [{"rule": {"expr": [
                {"counter": {"packets": self.dropped}}]}}]})
        return SimpleNamespace(returncode=status, stdout=output)

    def open(self, request, timeout):
        url = urlsplit(request.full_url)
        self.requests.append((request, self.resumed, self.budget))
        status, value = 200, None
        if url.port == 8229:
            authenticated = request.get_header("Authorization") == "Bearer public-admin-probe"
            if (authenticated and not self.authorized) or (not authenticated and self.denies_anonymous):
                status = 401
            elif url.path == "/":
                status = 404
            elif url.path == "/report":
                if self.resumed:
                    self.reports += 1
                    if self.replay == "ready" and self.reports == 3:
                        self.budget = 1234
                status = 401 if self.resumed and self.replay == "unauthorized" else (
                    503 if self.budget is None else 200)
                value = {"capacity_bytes": self.budget}
            elif url.path == "/sessions" and request.method == "POST":
                value = {"token": "public-session-probe"}
            elif url.path.startswith("/sessions/") and request.method == "DELETE":
                status = 204
            else:
                raise AssertionError("Unexpected administration operation")
        elif url.port == 7887:
            self.dropped += 1
            status = 502
        else:
            value = 0 if url.path == "/count" else "public listener"
        raw = b"" if value is None else json.dumps(value).encode()
        if status >= 400:
            raise HTTPError(request.full_url, status, "public probe response", {}, io.BytesIO(raw))
        response = io.BytesIO(raw)
        response.status = status
        return response

    def sleep(self, seconds):
        self.now += seconds

    @contextmanager
    def installed(self):
        with patch.dict(host.os.environ, {"CIPROXY_CACHE_ADMIN_TOKEN_FILE": self.token_file}), \
                patch.object(host.pwd, "getpwnam", return_value=self.account), \
                patch.object(host, "Path", self.path), \
                patch.object(lifecycle, "Path", self.path), \
                patch.object(lifecycle, "POLICY", self.path("/etc/ciproxy-cache.nft")), \
                patch.object(host.subprocess, "run", self.run), \
                patch("urllib.request.OpenerDirector.open", self.open), \
                patch.object(lifecycle, "host_listener", listener), \
                patch.object(lifecycle.socket, "create_connection", side_effect=OSError), \
                patch.object(lifecycle.time, "monotonic", lambda: self.now), \
                patch.object(lifecycle.time, "sleep", self.sleep):
            yield host.InstalledCache()


@contextmanager
def listener(family):
    yield 18080
