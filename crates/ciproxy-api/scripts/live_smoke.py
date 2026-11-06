#!/usr/bin/env python3
"""Verify an operator's live dashboard session without printing secret data."""

import json
import os
from pathlib import Path
import sys
from urllib.error import HTTPError, URLError
from urllib.parse import urlparse
from urllib.request import HTTPRedirectHandler, Request, build_opener


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, new_url):
        return None


def main() -> int:
    path = os.environ.get("CIPROXY_API_CREDENTIALS_FILE")
    if not path:
        print("Set CIPROXY_API_CREDENTIALS_FILE to a private JSON file.")
        return 2
    opener = build_opener(NoRedirect())
    try:
        config = json.loads(Path(path).read_text())
        origin = config["base_url"].rstrip("/")
        if urlparse(origin).scheme != "https":
            raise ValueError("HTTPS is required")
        cookie = config["session_cookie"]
        csrf = config["csrf_token"]
        workspace = config["workspace_id"]
    except (OSError, ValueError, KeyError, TypeError):
        print("Could not read the required private credential fields.")
        return 2

    def call(method: str, route: str, session: str = cookie) -> dict | list:
        headers = {"Cookie": "__Host-ciproxy=" + session}
        if method != "GET":
            headers["Origin"] = origin
            headers["X-Ciproxy-CSRF"] = csrf
        request = Request(origin + "/api" + route, headers=headers, method=method)
        with opener.open(request, timeout=30) as response:
            if not response.headers.get("x-ciproxy-build"):
                raise ValueError("missing build identity")
            return json.loads(response.read())

    try:
        session = call("GET", "/session")
        if not isinstance(session, dict) or session.get("state") != "ready":
            print("The supplied session has no private workspace access.")
            return 1
        previous = config.get("previous_session_cookie")
        if previous:
            old = call("GET", "/session", previous)
            if not isinstance(old, dict) or old["expires_at"] != session["expires_at"]:
                print("Repeated authorization changed the fixed sign-in expiry.")
                return 1
            print("PASS: repeated authorization preserves sign-in expiry")
        for route in ["/setup", "/machines", "/settings", "/members", "/labels", "/organizations"]:
            call("GET", "/workspaces/" + workspace + route)
            print("PASS: " + route)
        issued = call("POST", "/workspaces/" + workspace + "/install-tokens")
        if not isinstance(issued, dict) or not isinstance(issued.get("id"), str):
            raise ValueError("invalid issuance response")
        call("DELETE", "/workspaces/" + workspace + "/install-tokens/" + issued["id"])
        print("PASS: issue and revoke an install token")
        return 0
    except HTTPError as error:
        print("Live request failed with HTTP " + str(error.code) + ".")
    except (URLError, ValueError, KeyError, TypeError):
        print("Live verification could not finish. No response body was printed.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
