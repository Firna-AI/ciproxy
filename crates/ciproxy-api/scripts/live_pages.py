#!/usr/bin/env python3
"""Run page and Stripe route checks with an operator-owned credential path."""

import json
import os
from pathlib import Path
import sys
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode, urlparse
from urllib.request import HTTPRedirectHandler, Request, build_opener


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, new_url):
        return None


def outside_repository(value):
    path = Path(value).resolve()
    if path.is_relative_to(Path(__file__).resolve().parents[3]):
        raise ValueError("private files must stay outside the public repository")
    return path


def request(config, method, suffix, body=None, webhook=False):
    origin = config["base_url"].rstrip("/")
    prefix = "" if webhook else f"/api/workspaces/{config['workspace_id']}"
    headers = {}
    data = None
    if webhook:
        data = outside_repository(config["stripe_webhook_body_file"]).read_bytes()
        headers["Stripe-Signature"] = outside_repository(
            config["stripe_webhook_signature_file"]
        ).read_text().strip()
    else:
        headers["Cookie"] = f"__Host-ciproxy={config['session_cookie']}"
        if method != "GET":
            headers["Origin"] = origin
            headers["x-ciproxy-csrf"] = config["csrf_token"]
        if body is not None:
            data = json.dumps(body).encode()
            headers["Content-Type"] = "application/json"
    call = Request(origin + prefix + suffix, data=data, headers=headers, method=method)
    with build_opener(NoRedirect()).open(call, timeout=30) as response:
        if response.status != 200:
            raise ValueError("unexpected status")
        if not response.headers.get("x-ciproxy-build"):
            raise ValueError("missing build header")
        if response.headers.get("Cache-Control") != "no-store":
            raise ValueError("missing no-store header")
        result = json.load(response)
    print(f"PASS {method} {suffix.split('?')[0]}")
    return result


def save_result(config, value):
    path = outside_repository(config["private_result_file"])
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    os.fchmod(descriptor, 0o600)
    with os.fdopen(descriptor, "w") as output:
        json.dump(value, output)
    print("The result is in the configured private result file.")


def run():
    action = sys.argv[1] if len(sys.argv) == 2 else "help"
    if action in ["help", "--help", "-h"]:
        print("Actions: read, review, checkout, return, change, addons, portal, free, webhook.")
        return 0
    credential_path = os.environ.get("CIPROXY_API_CREDENTIALS_FILE")
    if not credential_path:
        print("Set CIPROXY_API_CREDENTIALS_FILE to a private JSON file.")
        return 2
    config = json.loads(outside_repository(credential_path).read_text())
    origin = config["base_url"].rstrip("/")
    if urlparse(origin).scheme != "https":
        raise ValueError("HTTPS is required")
    if action == "read":
        for route in ["/jobs/live", "/jobs/history", "/savings", "/billing", "/plans", "/banner"]:
            result = request(config, "GET", route)
            if route == "/jobs/history" and result["jobs"]:
                request(config, "GET", "/jobs/" + result["jobs"][0]["id"])
        for selection in ["this_month", "last_month"]:
            request(config, "GET", "/savings?range=" + selection)
        return
    if action == "review":
        selection = config["selection"]
        query = urlencode({"tier": selection["tier"], **selection["add_ons"]})
        save_result(config, request(config, "GET", "/checkout/review?" + query))
        return
    if action == "change":
        quote_id = config.get("quote_id")
        if quote_id is None:
            review = json.loads(outside_repository(config["private_result_file"]).read_text())
            quote_id = review["quote"]["id"]
        outcome = request(config, "POST", "/billing/change", {"quote_id": quote_id})
        save_result(config, {"quote": {"id": quote_id}, "outcome": outcome})
        return
    commands = {
        "checkout": ("POST", "/checkout", config.get("selection")),
        "return": ("POST", "/checkout/return", {"session_id": config.get("checkout_session_id")}),
        "addons": ("PUT", "/billing/add-ons", config.get("add_ons")),
        "portal": ("POST", "/billing/portal", None),
        "free": ("POST", "/billing/move-to-free", None),
    }
    if action == "webhook":
        request(config, "POST", "/hooks/stripe", webhook=True)
        return
    method, route, body = commands[action]
    save_result(config, request(config, method, route, body))


if __name__ == "__main__":
    try:
        sys.exit(run())
    except (KeyError, TypeError, ValueError, OSError, HTTPError, URLError):
        print("FAIL page verification. Check the private inputs and service state.")
        sys.exit(1)
