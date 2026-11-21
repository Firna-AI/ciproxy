#!/usr/bin/env python3
"""Prepare matching verified agent and cache assets for a disposable live check."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import urllib.error
import urllib.parse
import urllib.request


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, target):
        return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--assets", type=Path, required=True)
    parser.add_argument("--publish", action="store_true")
    args = parser.parse_args()
    credentials = json.loads(Path(os.environ["CIPROXY_SMOKE_CREDENTIALS"]).read_text())
    token = credentials["github_token"]
    headers = {"Authorization": "Bearer " + token, "Accept": "application/vnd.github+json",
               "User-Agent": "ciproxy-smoke-release", "X-GitHub-Api-Version": "2026-03-10"}
    opener = urllib.request.build_opener(NoRedirect())

    def request(method, url, value=None, raw=None, content_type=None):
        data = json.dumps(value).encode() if value is not None else raw
        supplied = dict(headers)
        supplied["Content-Type"] = content_type or "application/json"
        try:
            with opener.open(urllib.request.Request(url, data=data, headers=supplied, method=method), timeout=120) as response:
                result = response.read()
                return json.loads(result) if result else None
        except urllib.error.HTTPError as error:
            raise RuntimeError("GitHub release operation returned HTTP " + str(error.code)) from None
        except (OSError, ValueError):
            raise RuntimeError("GitHub release operation did not complete") from None

    files = []
    for binary in ["ciproxy-agent", "ciproxy-cache-sidecar"]:
        candidates = list(args.assets.glob(binary + "-*-x86_64-unknown-linux-gnu.tar.gz"))
        if len(candidates) != 1:
            raise RuntimeError("Provide exactly one verified archive for " + binary)
        archive = candidates[0]
        checksum = Path(str(archive) + ".sha256")
        expected = checksum.read_text().split()[0]
        digest = hashlib.sha256()
        with archive.open("rb") as source:
            for block in iter(lambda: source.read(1024 * 1024), b""):
                digest.update(block)
        if digest.hexdigest() != expected:
            raise RuntimeError("Release archive checksum does not match")
        files.extend([archive, checksum])
    api = "https://api.github.com/repos/" + args.repository
    release = request("POST", api + "/releases", {"tag_name": args.tag, "name": args.tag,
                      "draft": True, "prerelease": False})
    upload = release["upload_url"].split("{", 1)[0]
    for path in files:
        request("POST", upload + "?name=" + urllib.parse.quote(path.name, safe=""),
                raw=path.read_bytes(), content_type="application/octet-stream")
    if args.publish:
        request("PATCH", api + "/releases/" + str(release["id"]), {"draft": False})
    print(json.dumps({"release_id": release["id"], "tag": args.tag,
                      "draft": not args.publish, "assets": [path.name for path in files]}))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, RuntimeError) as error:
        print("Release preparation failed: " + str(error))
        raise SystemExit(1) from None
