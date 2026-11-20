"""Private input, image files, and a controlled clock for the smoke entries."""

from contextlib import contextmanager
import json
import os
from pathlib import PurePosixPath
from unittest.mock import patch

import own_smoke
import smoke_image


def index(newest, *available):
    return json.dumps({"newest": newest, "available": [
        {"image_id": name, "root_disk_bytes": 60_263_759_872} for name in available]})


class File:
    def __init__(self, host, path):
        self.host, self.path = host, PurePosixPath(path)

    def __truediv__(self, child):
        return File(self.host, self.path / child)

    def mkdir(self, **kwargs):
        pass

    def read_text(self):
        name = str(self.path)
        if name == "/test/index.json":
            value = self.host.snapshots[0]
            if isinstance(value, Exception):
                raise value
            return value
        if name == "/test/state.json":
            self.host.events.append("warm")
            return json.dumps({"runners": [{"report": {"state": {"state": "warm"}}}]})
        return self.host.files[name]


class ImageHost:
    def __init__(self, *snapshots):
        self.snapshots = list(snapshots)
        self.now = 0
        self.events = []
        self.files = {
            "/test/input.json": json.dumps({"public_url": "https://smoke.invalid",
                "workspace_id": "workspace", "machine_id": "machine", "deadline_seconds": 1,
                "repository": "owner/private-smoke", "ref": "main"}),
            "/test/credentials.json": json.dumps({"session_cookie": "public-test-cookie",
                                                   "github_token": "public-test-value"}),
        }

    def path(self, value):
        return File(self, value)

    def sleep(self, seconds):
        self.now += seconds
        if len(self.snapshots) > 1:
            self.snapshots.pop(0)
            if len(self.snapshots) == 1:
                self.events.append("published")

    def request(self, origin, method, path, headers, body=None, absent=False):
        if path == "/api/session":
            return {"state": "ready", "csrf_token": "public-test-csrf"}
        self.events.append("dispatch")
        raise StopAtDispatch

    @contextmanager
    def installed(self, **extra):
        environment = {"CIPROXY_OWN_SMOKE_INPUT": "/test/input.json",
            "CIPROXY_SMOKE_CREDENTIALS": "/test/credentials.json",
            "CIPROXY_SMOKE_STATE": "/test/state.json", "CIPROXY_SMOKE_IMAGE_INDEX": "/test/index.json"}
        environment.update(extra)
        with patch.dict(os.environ, environment, clear=True), \
                patch.object(smoke_image, "Path", self.path), \
                patch.object(own_smoke, "Path", self.path), \
                patch.object(smoke_image.time, "monotonic", lambda: self.now), \
                patch.object(smoke_image.time, "sleep", self.sleep), \
                patch.object(own_smoke.Smoke, "request", self.request):
            yield


class StopAtDispatch(Exception):
    """The test stops before it can start a real workflow."""
