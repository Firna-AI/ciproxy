"""Secret-safe GitHub App authentication for the standalone live probes."""

import base64
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

API = "https://api.github.com"
VERSION = "2026-03-10"
PERMISSIONS = {
    "organization_self_hosted_runners": "write",
    "metadata": "read",
    "actions": "read",
}
SCRATCH = Path(".context/ciproxy-mvp/milestone-6/live-probes")


class ProbeError(Exception):
    """An error whose message contains no response body or credential."""


def emit(event, **fields):
    print(json.dumps({"event": event, **fields}, sort_keys=True), flush=True)


def expect(status, allowed, label):
    if status not in allowed:
        raise ProbeError(f"{label}: HTTP {status}; expected {list(allowed)}")


def segment(value):
    return urllib.parse.quote(str(value), safe="")


def repository(value):
    parts = value.split("/")
    if len(parts) != 2 or not all(parts):
        raise ProbeError("Repository must have the form OWNER/NAME")
    return "/".join(segment(part) for part in parts)


def trusted_url(value):
    parsed = urllib.parse.urlsplit(value)
    host = parsed.hostname or ""
    allowed = host == "api.github.com" or host.endswith(".actions.githubusercontent.com")
    if parsed.scheme != "https" or not allowed or parsed.username or parsed.password:
        raise ProbeError("GitHub returned an unrecognized service origin")
    if parsed.port not in (None, 443) or parsed.fragment:
        raise ProbeError("GitHub returned an unrecognized service URL")
    return value


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def request(url, authorization, method="GET", body=None, headers=None, timeout=60):
    """Return status and decoded JSON. Never expose raw errors or bodies."""
    trusted_url(url)
    data = None if body is None else json.dumps(body).encode()
    merged = {"Accept": "application/vnd.github+json", "User-Agent": "ciproxy-live-probe"}
    merged.update({"X-GitHub-Api-Version": VERSION, "Authorization": authorization})
    if data is not None:
        merged["Content-Type"] = "application/json"
    merged.update(headers or {})
    req = urllib.request.Request(url, data=data, headers=merged, method=method)
    opener = urllib.request.build_opener(NoRedirect)
    try:
        with opener.open(req, timeout=timeout) as response:
            status, raw = response.status, response.read()
    except urllib.error.HTTPError as error:
        status = error.code
        error.close()
        return status, None
    except (OSError, urllib.error.URLError):
        raise ProbeError("HTTP transport failed; response and credentials withheld") from None
    if not raw or status == 202:
        return status, None
    try:
        return status, json.loads(raw.decode("utf-8-sig"))
    except (ValueError, UnicodeError):
        raise ProbeError("HTTP response was not JSON; body withheld") from None


def b64(value):
    return base64.urlsafe_b64encode(value).decode().rstrip("=")


class App:
    def __init__(self, org):
        path = os.environ.get("CIPROXY_GITHUB_APP_PATH")
        if not path:
            raise ProbeError("Set CIPROXY_GITHUB_APP_PATH to the App JSON file with id and pem")
        try:
            credentials = json.loads(Path(path).read_text())
            self.issuer = str(credentials["id"])
            self.pem = credentials["pem"].encode()
            if not self.issuer or not self.pem:
                raise ValueError()
        except (OSError, ValueError, KeyError, TypeError, AttributeError):
            raise ProbeError("Cannot read App JSON id and pem; file contents withheld") from None
        self.org = org
        self.token = None
        self.minted_at = 0
        for attempt in range(8):
            status, installation = request(
                f"{API}/orgs/{segment(org)}/installation", "Bearer " + self.jwt())
            if status != 404 or attempt == 7:
                break
            time.sleep(2)
        expect(status, (200,), "Find organization App installation")
        if installation.get("account", {}).get("type") != "Organization":
            raise ProbeError("The App installation must belong to an organization")
        self.installation_id = int(installation["id"])
        self.refresh()

    def jwt(self):
        now = int(time.time())
        header = b64(json.dumps({"alg": "RS256", "typ": "JWT"}).encode())
        claims = b64(json.dumps({"iss": self.issuer, "iat": now - 60, "exp": now + 480}).encode())
        unsigned = (header + "." + claims).encode()
        SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryFile(dir=SCRATCH) as key:
            key.write(self.pem)
            key.flush()
            key.seek(0)
            result = subprocess.run(
                ["openssl", "dgst", "-sha256", "-sign", f"/dev/fd/{key.fileno()}"],
                input=unsigned, capture_output=True, pass_fds=(key.fileno(),), check=False)
        if result.returncode:
            raise ProbeError("OpenSSL could not sign the App JWT; details withheld")
        return unsigned.decode() + "." + b64(result.stdout)

    def refresh(self):
        status, access = request(
            f"{API}/app/installations/{self.installation_id}/access_tokens",
            "Bearer " + self.jwt(), "POST", {"permissions": PERMISSIONS})
        expect(status, (201,), "Create installation token with three permissions")
        if access.get("permissions") != PERMISSIONS:
            raise ProbeError("Installation token did not return the exact three requested permissions")
        self.token = access["token"]
        self.minted_at = time.monotonic()
        emit("app_permissions", permissions=PERMISSIONS, installation_id=self.installation_id)

    def call(self, path, method="GET", body=None):
        if time.monotonic() - self.minted_at > 3000:
            self.refresh()
        return request(API + path, "Bearer " + self.token, method, body)

    def visible_private_repositories(self, names):
        wanted = {name.lower() for name in names}
        found = {}
        page = 1
        while wanted - found.keys():
            status, response = self.call(f"/installation/repositories?per_page=100&page={page}")
            expect(status, (200,), "List App installation repositories")
            rows = response["repositories"]
            for row in rows:
                name = row["full_name"].lower()
                if name in wanted:
                    if row.get("private") is not True:
                        raise ProbeError("Every fixture repository must be private")
                    found[name] = row
            if len(rows) < 100:
                break
            page += 1
        if wanted - found.keys():
            raise ProbeError("The App must list every fixture repository in its installation")
        emit("app_repository_visibility", repositories=[
            {"id": found[name]["id"], "full_name": found[name]["full_name"], "private": True}
            for name in sorted(wanted)])
        return found


class Cleanup:
    """Run every cleanup step without replacing an earlier body exception."""

    def __init__(self):
        self.steps = []

    def add(self, label, callback, priority=0):
        self.steps.append((priority, len(self.steps), label, callback))

    def __enter__(self):
        return self

    def __exit__(self, kind, error, traceback):
        failed = []
        for _, _, label, callback in sorted(self.steps, reverse=True):
            try:
                callback()
                emit("cleanup_pass", resource=label)
            except BaseException as failure:
                failed.append(label)
                detail = str(failure) if isinstance(failure, ProbeError) else type(failure).__name__
                emit("cleanup_failed", resource=label, detail=detail, **safe_error_context(failure))
        if failed and kind is None:
            raise ProbeError("Cleanup incomplete for " + ", ".join(failed)) from None


def safe_error_context(error):
    result = {"error_type": type(error).__name__}
    frame = error.__traceback__
    while frame is not None and frame.tb_next is not None:
        frame = frame.tb_next
    if frame is not None:
        result["location"] = {"file": Path(frame.tb_frame.f_code.co_filename).name,
                              "function": frame.tb_frame.f_code.co_name, "line": frame.tb_lineno}
    return result


def entrypoint(main):
    def interrupt(signum, frame):
        raise KeyboardInterrupt()

    signal.signal(signal.SIGTERM, interrupt)
    try:
        main()
    except KeyboardInterrupt as error:
        emit("failed", detail="Interrupted; registered cleanup steps ran", **safe_error_context(error))
        sys.exit(130)
    except ProbeError as error:
        emit("failed", detail=str(error), **safe_error_context(error))
        sys.exit(1)
    except Exception as error:
        emit("failed", detail=f"Unexpected local failure: {type(error).__name__}; details withheld",
             **safe_error_context(error))
        sys.exit(1)
