"""Process and HTTP helpers for manual executable verification."""

import json
from pathlib import Path
import socket
import subprocess
import time
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


ROOT = Path(__file__).resolve().parents[3]


def request(url, method="GET", body=None, headers=None):
    headers = dict(headers or {})
    if isinstance(body, dict):
        body = json.dumps(body).encode()
        headers["Content-Type"] = "application/json"
    try:
        response = urlopen(Request(url, data=body, headers=headers, method=method), timeout=30)
    except HTTPError as error:
        response = error
    with response:
        return response.status, response.read(), dict(response.headers)


def rpc(origin, operation, body):
    status, contents, _ = request(
        origin + "/twirp/github.actions.results.api.v1.CacheService/" + operation,
        "POST", body)
    assert status == 200, (operation, status, contents)
    return json.loads(contents)


def port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


class Processes:
    def __init__(self, directory):
        self.directory = directory
        directory.mkdir(parents=True, exist_ok=True)
        self.children = []
        self.logs = []

    def start(self, name, arguments, ready=None):
        output = open(self.directory / f"{name}.log", "w")
        child = subprocess.Popen(arguments, cwd=ROOT, stdout=output, stderr=subprocess.STDOUT)
        self.children.append(child)
        self.logs.append(output)
        if ready:
            deadline = time.monotonic() + 15
            while True:
                if child.poll() is not None:
                    raise RuntimeError(f"{name} exited; see {output.name}")
                try:
                    request(ready)
                    break
                except (URLError, ConnectionError):
                    if time.monotonic() >= deadline:
                        raise RuntimeError(f"{name} listener unavailable; see {output.name}")
                    time.sleep(0.02)
        return child

    def close(self):
        for child in reversed(self.children):
            child.terminate()
        for child in reversed(self.children):
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()
        for output in self.logs:
            output.close()


def start_cache(processes, registration, capacity=268435456):
    binary = ROOT / "target/debug/ciproxy-cache-sidecar"
    if not binary.exists():
        raise RuntimeError("run cargo build --package ciproxy-cache-sidecar first")
    machine = f"http://127.0.0.1:{port()}"
    admin = f"http://127.0.0.1:{port()}"
    proxy = f"http://127.0.0.1:{port()}"
    token_file = processes.directory / "admin-token"
    token_file.write_text("manual-smoke-admin-secret")
    token_file.chmod(0o600)
    processes.start("machine", [str(binary), "machine", "--listen", machine.removeprefix("http://"),
                               "--admin-listen", admin.removeprefix("http://"), "--cache-dir",
                               str(processes.directory / "cache"), "--capacity-bytes", str(capacity),
                               "--admin-token-file", str(token_file)], admin + "/report")
    registration = dict(registration, proxy_url=proxy)
    status, contents, _ = request(admin + "/sessions", "POST", registration,
                                 {"Authorization": "Bearer manual-smoke-admin-secret"})
    assert status == 200, contents
    credential = json.loads(contents)
    config = processes.directory / "proxy.json"
    config.write_text(json.dumps({"machine_url": machine, "machine_token": credential["token"],
                                  "registration": registration}))
    config.chmod(0o600)
    processes.start("proxy", [str(binary), "proxy", "--listen", proxy.removeprefix("http://"),
                             "--config", str(config)], proxy + "/_smoke/ready")
    return {"machine": machine, "admin": admin, "proxy": proxy, "directory": str(processes.directory),
            "scope": credential["scope"]}
