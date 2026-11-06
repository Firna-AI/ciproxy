#!/usr/bin/env python3
"""Prepare and verify GitHub install binding with private operator inputs."""

import argparse
import hashlib
import hmac
import json
import os
from pathlib import Path
import secrets
import sys
from urllib.error import HTTPError, URLError
from urllib.parse import parse_qs, urlencode, urlparse
from urllib.request import HTTPRedirectHandler, Request, build_opener


def private_write(path: Path, value: dict) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    os.fchmod(descriptor, 0o600)
    with os.fdopen(descriptor, "w") as output:
        json.dump(value, output)


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, new_url):
        return None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=["prepare", "verify"])
    arguments = parser.parse_args()
    config_path = os.environ.get("CIPROXY_API_CREDENTIALS_FILE")
    if not config_path:
        print("Set CIPROXY_API_CREDENTIALS_FILE to a private JSON file.")
        return 2
    opener = build_opener(NoRedirect())
    try:
        config = json.loads(Path(config_path).read_text())
        proof_path = Path(config["installation_proof_file"])
        user_token = config["github_user_token"]
        app_jwt = config["github_app_jwt"]
        target = int(config["target_organization_id"])

        def get(path: str, token: str):
            request = Request("https://api.github.com/" + path, headers={
                "Authorization": "Bearer " + token,
                "Accept": "application/vnd.github+json",
                "X-GitHub-Api-Version": "2026-03-10",
                "User-Agent": "ciproxy-api-live-proof",
            })
            with opener.open(request, timeout=30) as response:
                return json.loads(response.read())

        profile = get("user", user_token)
        if arguments.mode == "prepare":
            state = secrets.token_urlsafe(32)
            query = urlencode({"state": state})
            private_write(proof_path, {
                "state": state,
                "requester_id": profile["id"],
                "target_id": target,
                "install_url": "https://github.com/apps/" + config["github_app_slug"]
                    + "/installations/new?" + query,
            })
            print("Prepared the install URL and expected state in the private proof file.")
            print("Open that URL in the initiating member or owner's GitHub browser.")
            return 0

        proof = json.loads(proof_path.read_text())
        if profile["id"] != proof["requester_id"] or target != proof["target_id"]:
            print("FAIL: the operator changed the initiating identity or target")
            return 1
        setup_url = urlparse(proof["observed_setup_url"])
        if setup_url.scheme != "https" or setup_url.path != "/api/github/setup":
            print("FAIL: the observed return is not the registered HTTPS setup route")
            return 1
        callback = parse_qs(setup_url.query)
        if callback.get("state") != [proof["state"]]:
            print("FAIL: GitHub did not return the initiating flow state")
            return 1
        print("PASS: setup callback state matches its initiating flow")
        proven = False
        pending = []
        for page in range(1, 1000000):
            batch = get(f"app/installation-requests?per_page=100&page={page}", app_jwt)
            pending.extend(request for request in batch
                if request["requester"]["id"] == proof["requester_id"]
                and request["account"]["id"] == target)
            if len(batch) < 100:
                break
        if pending:
            proven = True
            print("PASS: the App identifies the pending request's member and organization")
        installation_ids = callback.get("installation_id", [])
        if installation_ids:
            allowed = []
            for page in range(1, 1000000):
                batch = get(f"user/installations?per_page=100&page={page}", user_token)["installations"]
                allowed.extend(installation for installation in batch
                    if installation["account"]["id"] == target
                    and installation["account"]["type"] == "Organization")
                if len(batch) < 100:
                    break
            if not any(str(item["id"]) == installation_ids[0] for item in allowed):
                print("FAIL: the callback installation is not verified for the initiating person")
                return 1
            installed = get("app/installations/" + installation_ids[0], app_jwt)
            if (str(installed["id"]) != installation_ids[0]
                    or installed["account"]["id"] != target
                    or installed["account"]["type"] != "Organization"):
                print("FAIL: configured App and own-user installation identity differ")
                return 1
            proven = True
            print("PASS: configured App and own-user token verify the callback installation")
        if "webhook_body_file" in config:
            raw = Path(config["webhook_body_file"]).read_bytes()
            expected = "sha256=" + hmac.new(config["webhook_secret"].encode(), raw, hashlib.sha256).hexdigest()
            if not hmac.compare_digest(expected, config["webhook_signature"]):
                print("FAIL: captured installation webhook signature")
                return 1
            event = json.loads(raw)
            if event.get("action") != "created" or event["installation"]["account"].get("type") != "Organization":
                print("FAIL: captured webhook is not a created organization installation")
                return 1
            actor = event.get("requester") or event["sender"]
            if actor["id"] != proof["requester_id"]:
                print("FAIL: approval did not retain the initiating requester")
                return 1
            if event["installation"]["account"]["id"] != target:
                print("FAIL: approval named a different organization")
                return 1
            proven = True
            print("PASS: signed approval retains the initiating requester and organization")
        if proven and not installation_ids:
            print("WAITING: request/event facts alone cannot authorize a workspace link")
        if not proven:
            print("FAIL: no verified request, installation, or signed approval proves the target")
            return 1
        return 0
    except HTTPError as error:
        print("Live GitHub request failed with HTTP " + str(error.code) + ".")
    except (OSError, URLError, ValueError, KeyError, TypeError):
        print("Live binding verification could not finish. No private data was printed.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
