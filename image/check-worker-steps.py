"""Run the installed Linux guest on the Worker steps smoke workflow."""

import argparse
import http.server
import json
import pathlib
import secrets
import ssl
import subprocess
import sys
import threading
import urllib.parse


HOST = "127.0.0.1"
MAX_REPORT_BYTES = 1024 * 1024
EXPECTED = (
    ("Check out smoke repository", "ran"),
    ("Run marker", "ran"),
    ("Skip marker", "skipped"),
    ("Composite wrapper", "ran"),
    ("Last marker", "ran"),
    ("Post Check out smoke repository", "ran"),
)


class CheckFailed(Exception):
    """A failure message that contains no report or registration data."""


class Collector(http.server.ThreadingHTTPServer):
    def __init__(self, url, token):
        self.path = url.path or "/"
        self.bearer = f"Bearer {token}".encode()
        self.report = None
        super().__init__((HOST, url.port or 443), Callback)


class Callback(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        auth = self.headers.get("Authorization", "").encode()
        if self.path != self.server.path or not secrets.compare_digest(auth, self.server.bearer):
            self.respond(403)
            return
        try:
            length = int(self.headers.get("Content-Length", ""))
            if not 0 < length <= MAX_REPORT_BYTES:
                raise ValueError
            message = json.loads(self.rfile.read(length))
            if message["type"] == "runner_exited":
                self.server.report = message
            elif message["type"] != "usage_samples":
                raise ValueError
        except (KeyError, ValueError, TypeError):
            self.respond(400)
            return
        self.respond(204)

    def respond(self, status):
        self.send_response(status)
        self.end_headers()


def check_report(report, runner_id):
    if report is None:
        raise CheckFailed("The guest did not send a final report.")
    if report["runner_id"] != runner_id or report["exit"]["code"] != 0:
        raise CheckFailed("The final report does not confirm a clean exit of this runner.")
    usage = report.get("usage")
    if not usage or not usage["cpu_milli_vcpu"] or not usage["peak_memory_mib"]:
        raise CheckFailed("The final report has no usage samples.")
    steps = usage["steps"]
    if tuple((step["name"], step["status"]) for step in steps) != EXPECTED:
        raise CheckFailed("The Worker steps do not match the smoke workflow.")
    previous_end = 0
    for step in steps:
        if step["status"] == "skipped":
            if "started_at_ms" in step or "ended_at_ms" in step:
                raise CheckFailed("A skipped step has a time.")
            continue
        started = step.get("started_at_ms")
        ended = step.get("ended_at_ms")
        if (type(started) is not int or type(ended) is not int
                or not previous_end <= started <= ended or started % 1000 or ended % 1000):
            raise CheckFailed("A step lacks ordered times in whole seconds.")
        previous_end = ended


def run_check(args):
    registration = json.loads(args.registration_file.read_text())
    url = urllib.parse.urlsplit(registration["exit_report_url"])
    if (url.scheme != "https" or url.hostname != HOST or url.username
            or url.password or url.query or url.fragment):
        raise CheckFailed("Use a loopback HTTPS callback URL with no credentials or query.")
    tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    tls.load_cert_chain(args.tls_cert, args.tls_key)
    with Collector(url, registration["exit_report_token"]) as server:
        server.socket = tls.wrap_socket(server.socket, server_side=True)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            subprocess.run(
                [args.guest, "--image-id", "ciproxy-worker-steps-smoke", "boat",
                 "--registration-file", args.registration_file],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True, timeout=600,
            )
        finally:
            server.shutdown()
            thread.join()
        check_report(server.report, registration["configure"]["runner_id"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--guest", default="/usr/local/bin/ciproxy-guest")
    parser.add_argument("--registration-file", type=pathlib.Path, required=True)
    parser.add_argument("--tls-cert", type=pathlib.Path, required=True)
    parser.add_argument("--tls-key", type=pathlib.Path, required=True)
    args = parser.parse_args()
    try:
        run_check(args)
    except CheckFailed as error:
        print(error, file=sys.stderr)
        return 1
    except subprocess.TimeoutExpired:
        print("The smoke guest did not exit. Check that the job was queued.", file=sys.stderr)
        return 1
    except (OSError, KeyError, TypeError, ValueError, subprocess.CalledProcessError):
        print("The worker step check could not run or read the report.", file=sys.stderr)
        return 1
    print("Worker steps match the smoke workflow.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
