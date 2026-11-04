#!/usr/bin/env python3
"""No-secret cache v1/v2 and signed-blob fixture for integration checks."""

import argparse
import base64
import hashlib
import json
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse
from xml.etree import ElementTree


class State:
    def __init__(self):
        self.lock = threading.RLock()
        self.entries = {}
        self.uploads = {}
        self.calls = []
        self.options = {"lookup": True, "failure": False}


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_args):
        pass

    def read_body(self):
        if self.headers.get("Transfer-Encoding", "").lower() == "chunked":
            result = bytearray()
            while True:
                size = int(self.rfile.readline().split(b";", 1)[0], 16)
                if not size:
                    while self.rfile.readline() not in (b"\r\n", b"\n", b""):
                        pass
                    return bytes(result)
                result.extend(self.rfile.read(size))
                self.rfile.read(2)
        return self.rfile.read(int(self.headers.get("Content-Length", "0")))

    def send(self, code, body=b"", headers=None):
        if isinstance(body, dict):
            body = json.dumps(body).encode()
        self.send_response(code)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Content-Type", "application/json")
        for key, value in (headers or {}).items():
            self.send_header(key, value)
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(body)

    def handle_request(self):
        body = self.read_body()
        url = urlparse(self.path)
        query = parse_qs(url.query)
        state = self.server.state
        with state.lock:
            if url.path == "/_fixture/state":
                return self.send(200, {"calls": state.calls, "entries": [
                    {"key": key, "version": version, "size": len(data),
                     "sha256": hashlib.sha256(data).hexdigest()}
                    for (key, version), data in state.entries.items()]})
            if url.path == "/_fixture/options":
                state.options.update(json.loads(body))
                return self.send(200, {})
            if url.path == "/_fixture/entries":
                state.entries.clear()
                return self.send(200, {})
            if url.path == "/_fixture/seed":
                seed = json.loads(body)
                state.entries[(seed["key"], seed["version"])] = base64.b64decode(seed["base64"])
                return self.send(200, {})
            state.calls.append({"method": self.command, "path": self.path,
                                "authorization": self.headers.get("Authorization"),
                                "size": len(body)})
            if state.options["failure"]:
                return self.send(503, {"msg": "fixture failure"})
            if url.path.startswith("/signed/"):
                return self.blob(url.path, query, body, state)
            if "CacheService/" in url.path:
                return self.rpc(url.path.rsplit("/", 1)[1], json.loads(body), state)
            if "_apis/artifactcache/" in url.path:
                resource = url.path.split("_apis/artifactcache/", 1)[1]
                return self.legacy(resource, query, body, state)
            return self.send(202, body, {"X-Fixture": "results-pass-through"})

    def reserve(self, request, state):
        identifier = str(len(state.uploads) + 1)
        state.uploads[identifier] = {"key": request["key"], "version": request["version"],
                                     "data": b"", "blocks": {}, "ranges": {}}
        return identifier

    def cache_response(self, body):
        if self.server.state.options.get("snake"):
            names = {"signedUploadUrl": "signed_upload_url", "signedDownloadUrl": "signed_download_url",
                     "entryId": "entry_id", "matchedKey": "matched_key"}
            body = {names.get(key, key): value for key, value in body.items()}
        return self.send(200, body)

    def rpc(self, operation, request, state):
        if operation == "CreateCacheEntry":
            identifier = self.reserve(request, state)
            return self.cache_response({"ok": True, "signedUploadUrl":
                                  f"{self.server.origin}/signed/{identifier}?sig=fixture-secret"})
        if operation == "FinalizeCacheEntryUpload":
            upload = next(item for item in reversed(list(state.uploads.values()))
                          if item["key"] == request["key"] and item["version"] == request["version"])
            state.entries[(upload["key"], upload["version"])] = upload["data"]
            return self.cache_response({"ok": True, "entryId": "100"})
        keys = [request["key"], *request.get("restoreKeys", request.get("restore_keys", []))]
        for key in keys:
            matches = [(candidate, data) for (candidate, version), data in state.entries.items()
                       if candidate.startswith(key) and version == request["version"]]
            if matches and state.options["lookup"]:
                matched, data = next((item for item in matches if item[0] == key), matches[-1])
                identifier = self.reserve({"key": matched, "version": request["version"]}, state)
                state.uploads[identifier]["data"] = data
                return self.cache_response({"ok": True, "matchedKey": matched, "signedDownloadUrl":
                                      f"{self.server.origin}/signed/{identifier}?sig=fixture-secret"})
        return self.cache_response({"ok": False})

    def legacy(self, resource, query, body, state):
        if resource == "cache":
            request = {"key": query["keys"][0].split(",")[0],
                       "restoreKeys": query["keys"][0].split(",")[1:], "version": query["version"][0]}
            for key in [request["key"], *request["restoreKeys"]]:
                for (matched, version), data in reversed(list(state.entries.items())):
                    if state.options["lookup"] and version == request["version"] and matched.startswith(key):
                        identifier = self.reserve({"key": matched, "version": version}, state)
                        state.uploads[identifier]["data"] = data
                        return self.send(200, {"cacheKey": matched, "archiveLocation":
                                              f"{self.server.origin}/signed/{identifier}?sig=fixture-secret"})
            return self.send(204)
        if resource == "caches":
            return self.send(200, {"cacheId": int(self.reserve(json.loads(body), state))})
        upload = state.uploads[resource.split("/")[-1]]
        if self.command == "PATCH":
            start = int(self.headers["Content-Range"].split(" ")[1].split("-")[0])
            upload["ranges"][start] = body
            return self.send(204)
        upload["data"] = b"".join(data for _, data in sorted(upload["ranges"].items()))
        state.entries[(upload["key"], upload["version"])] = upload["data"]
        return self.send(204)

    def blob(self, path, query, body, state):
        if query.get("sig") != ["fixture-secret"] or self.headers.get("Authorization"):
            return self.send(403, {"msg": "signed URL or header mismatch"})
        upload = state.uploads[path.split("/")[-1]]
        if self.command == "PUT":
            if query.get("comp") == ["block"]:
                upload["blocks"][query["blockid"][0]] = body
            elif query.get("comp") == ["blocklist"]:
                upload["data"] = b"".join(upload["blocks"][block.text]
                                           for block in ElementTree.fromstring(body))
            else:
                upload["data"] = body
            return self.send(201, headers={"ETag": '"fixture"'})
        return self.send(200, upload["data"], {"ETag": '"fixture"', "Accept-Ranges": "bytes"})

    do_GET = handle_request
    do_HEAD = handle_request
    do_POST = handle_request
    do_PUT = handle_request
    do_PATCH = handle_request
    do_DELETE = handle_request


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=0)
    args = parser.parse_args()
    server = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
    server.state = State()
    server.origin = f"http://127.0.0.1:{server.server_port}"
    print(server.origin, flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
