#!/usr/bin/env python3
"""Exercise real sidecar executables with a 50 MiB fake-GitHub archive."""

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import signal
import sys
import tempfile
from urllib.parse import urlencode

sys.dont_write_bytecode = True

from runtime import ROOT, Processes, port, request, rpc, start_cache


def exercise(environment, github):
    proxy = environment["proxy"]
    archive = os.urandom(50 * 1024 * 1024)
    create = rpc(proxy, "CreateCacheEntry", {"key": "smoke-50m", "version": "wire-v1"})
    upload = create["signedUploadUrl"]
    blocks = [archive[index:index + 8 * 1024 * 1024] for index in range(0, len(archive), 8 * 1024 * 1024)]
    order = [base64.b64encode(index.to_bytes(4, "big")).decode() for index in range(len(blocks))]
    for index in reversed(range(len(blocks))):
        url = upload + "?" + urlencode({"comp": "block", "blockid": order[index]})
        assert request(url, "PUT", blocks[index])[0] == 201
    assert request(upload + "?" + urlencode({"comp": "block", "blockid": order[0]}), "PUT", blocks[0])[0] == 201
    blocklist = ("<BlockList>" + "".join(f"<Latest>{block}</Latest>" for block in order) + "</BlockList>").encode()
    assert request(upload + "?comp=blocklist", "PUT", blocklist)[0] == 201
    assert rpc(proxy, "FinalizeCacheEntryUpload", {"key": "smoke-50m", "version": "wire-v1", "sizeBytes": str(len(archive))}) == {"ok": True, "entryId": "100"}
    lookup = rpc(proxy, "GetCacheEntryDownloadURL", {"key": "smoke-50m", "version": "wire-v1", "restoreKeys": []})
    admin_headers = {"Authorization": "Bearer manual-smoke-admin-secret"}
    report = json.loads(request(environment["admin"] + "/report", headers=admin_headers)[1])
    assert report["used_bytes"] == len(archive), report
    assert report["restores_last_7_days"] == 0, report
    request(github + "/_fixture/entries", "DELETE")
    status, downloaded, _ = request(lookup["signedDownloadUrl"])
    assert status == 200
    assert hashlib.sha256(downloaded).digest() == hashlib.sha256(archive).digest()
    status, partial, _ = request(lookup["signedDownloadUrl"], headers={"Range": "bytes=7-19"})
    assert status == 206 and partial == archive[7:20]
    report = json.loads(request(environment["admin"] + "/report", headers=admin_headers)[1])
    assert report["restores_last_7_days"] == 1 and report["hits_last_7_days"] == 1, report
    assert request(proxy + "/twirp/ArtifactService/UploadLog", "POST", b"unchanged-results")[0:2] == (202, b"unchanged-results")
    assert request(environment["admin"] + "/clear", "POST", headers=admin_headers)[0] == 204
    assert not rpc(proxy, "GetCacheEntryDownloadURL", {"key": "smoke-50m", "version": "wire-v1"})["ok"]
    print("PASS: 50 MiB staged upload, retry, block order, retained restore, range, counts, results, clear", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--serve", action="store_true", help="keep the fixture and proxies up for SDK checks")
    args = parser.parse_args()
    context = ROOT / ".context/ciproxy-mvp-m11"
    context.mkdir(parents=True, exist_ok=True)
    directory = Path(tempfile.mkdtemp(prefix="executable-smoke-", dir=context))
    processes = Processes(directory)
    try:
        github = f"http://127.0.0.1:{port()}"
        processes.start("github", [sys.executable, str(Path(__file__).with_name("fake-github.py")), "--port", github.rsplit(":", 1)[1]], github + "/_fixture/state")
        registration = {"job_id": "manual-smoke", "facts": {"organization_id": "1", "repository": {"id": "10", "full_name": "example/repository"},
                         "git_ref": "refs/heads/main", "event": "push", "head_repository": None,
                         "default_ref": "refs/heads/main", "base_ref": None}, "permissions": {"read": True, "write": True},
                         "github_token": "fake-runtime-token", "cache_url": github + "/legacy-prefix/", "results_url": github}
        environment = start_cache(processes, registration)
        exercise(environment, github)
        environment["github"] = github
        (directory / "environment.json").write_text(json.dumps(environment, indent=2))
        print(json.dumps(environment), flush=True)
        if args.serve:
            print("Ready for SDK checks. Stop with Ctrl-C.", flush=True)
            signal.pause()
    finally:
        processes.close()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
