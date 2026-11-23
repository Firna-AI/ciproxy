"""Regression coverage for static export aliases, headers, and staging."""

import io
import json
from contextlib import redirect_stdout
from email import policy
from email.parser import BytesParser
from pathlib import Path
import subprocess
import sys
import tempfile
from threading import Event
import unittest
from unittest.mock import patch
from urllib.error import HTTPError
from urllib.parse import parse_qs, unquote, urlsplit

from deploy.scripts import publish_static


def request_name(request):
    headers = (
        f"Content-Type: {request.get_header('Content-type')}\r\nMIME-Version: 1.0\r\n\r\n"
    ).encode()
    message = BytesParser(policy=policy.default).parsebytes(headers + request.data)
    return json.loads(next(message.iter_parts()).get_payload(decode=True))["name"]


class StaticPublishingTests(unittest.TestCase):
    def test_dry_run_lists_route_aliases_and_exact_headers(self):
        publisher = Path(__file__).resolve().parents[1] / "scripts" / "publish_static.py"
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            for name, content in {
                "index.html": "<!doctype html><title>Home</title>",
                "docs/index.html": "<!doctype html><title>Docs</title>",
                "jobs.html": "<!doctype html><title>Jobs</title>",
                "assets/site.css": "body { color: black; }",
            }.items():
                path = directory / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8")
            result = subprocess.run(
                [sys.executable, str(publisher), "--directory", str(directory),
                 "--bucket", "dry-run", "--dry-run"],
                check=True, capture_output=True, text=True,
                env={"PATH": "", "LC_ALL": "C", "TZ": "UTC"},
            )
        published = json.loads(result.stdout)
        html_names = [
            "docs", "docs/", "docs/index.html", "index.html",
            "jobs", "jobs/", "jobs.html",
        ]
        self.assertCountEqual(
            [item["name"] for item in published],
            ["assets/site.css", *html_names],
        )
        by_name = {item["name"]: item for item in published}
        for name in html_names:
            with self.subTest(name=name):
                self.assertEqual(by_name[name]["contentType"], "text/html; charset=utf-8")
                self.assertEqual(by_name[name]["cacheControl"], "no-store")
        self.assertEqual(by_name["assets/site.css"]["contentType"], "text/css")
        self.assertEqual(
            by_name["assets/site.css"]["cacheControl"],
            "public,max-age=86400,must-revalidate",
        )

    def test_multipart_upload_stores_exact_cache_metadata_for_html_and_assets(self):
        scratch = Path(__file__).resolve().parents[2] / ".context/static-upload-tests"
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=scratch) as temporary:
            for name, filename, expected in (
                ("jobs", "jobs.html", {"name": "jobs", "contentType": "text/html; charset=utf-8", "cacheControl": "no-store"}),
                ("assets/site.css", "site.css", {"name": "assets/site.css", "contentType": "text/css", "cacheControl": "public,max-age=86400,must-revalidate"}),
            ):
                with self.subTest(filename=filename):
                    path = Path(temporary) / filename
                    path.write_bytes(b"offline-payload")
                    with patch.object(publish_static.urllib.request, "urlopen", return_value=io.BytesIO(b"{}")) as http:
                        publish_static.upload("private-static-origin", name, path, "synthetic-token")
                    request = http.call_args.args[0]
                    headers = ("Content-Type: " + request.get_header("Content-type") + "\r\nMIME-Version: 1.0\r\n\r\n").encode()
                    message = BytesParser(policy=policy.default).parsebytes(headers + request.data)
                    parts = list(message.iter_parts())
                    self.assertEqual(json.loads(parts[0].get_payload(decode=True)), expected)
                    # Cloud Storage refuses a media part whose type differs from the metadata.
                    media_part = f"Content-Type: {expected['contentType']}\r\n\r\n".encode()
                    self.assertIn(media_part + b"offline-payload", request.data)
                    self.assertEqual(parts[1].get_payload(decode=True), b"offline-payload")
                    self.assertEqual(request.get_method(), "POST")
                    self.assertEqual(request.full_url, "https://storage.googleapis.com/upload/storage/v1/b/private-static-origin/o?uploadType=multipart")

    def stage_export(self, names, open_url):
        scratch = Path(__file__).resolve().parents[2] / ".context/static-upload-tests"
        scratch.mkdir(parents=True, exist_ok=True)
        output = io.StringIO()
        with tempfile.TemporaryDirectory(dir=scratch) as temporary:
            directory = Path(temporary)
            for name in names:
                path = directory / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("offline-payload", encoding="utf-8")
            arguments = ["publish_static.py", f"--directory={directory}", "--bucket=app-bucket", "--new-only"]
            with (patch.object(sys, "argv", arguments), redirect_stdout(output),
                  patch.object(publish_static.subprocess, "check_output", return_value="synthetic-token"),
                  patch.object(publish_static.urllib.request, "urlopen", side_effect=open_url)):
                publish_static.main()
        return output.getvalue()

    def test_new_only_creates_missing_assets_before_html_and_aliases(self):
        existing = {"assets/old.js", "index.html", "jobs/"}
        upload_requests = []
        reads = []
        written = []

        def open_url(request, timeout):
            url = urlsplit(request.full_url)
            if request.get_method() == "GET":
                name = unquote(url.path.partition("/o/")[2])
                reads.append((request.full_url, request.get_header("Authorization")))
                if name in existing:
                    return io.BytesIO(json.dumps({"name": name}).encode())
                raise HTTPError(request.full_url, 404, "Not Found", {}, io.BytesIO())
            name = request_name(request)
            query = parse_qs(url.query)
            upload_requests.append((name, query))
            if name in existing and query.get("ifGenerationMatch") == ["0"]:
                raise HTTPError(request.full_url, 412, "Precondition Failed", {}, io.BytesIO())
            written.append(name)
            return io.BytesIO(b"{}")

        output = self.stage_export(
            ("index.html", "continue.html", "jobs.html", "assets/old.js", "assets/new.js"), open_url,
        )

        for name, query in upload_requests:
            self.assertEqual(query, {"uploadType": ["multipart"], "ifGenerationMatch": ["0"]}, name)
        names = [name for name, query in upload_requests]
        self.assertCountEqual(names[:2], ["assets/new.js", "assets/old.js"])
        self.assertCountEqual(names[2:], [
            "continue", "continue/", "continue.html", "index.html", "jobs", "jobs/", "jobs.html",
        ])
        self.assertCountEqual(written, [
            "assets/new.js", "continue", "continue/", "continue.html", "jobs", "jobs.html",
        ])
        metadata_url = "https://storage.googleapis.com/storage/v1/b/app-bucket/o"
        self.assertCountEqual(reads, [
            (f"{metadata_url}/assets%2Fold.js?fields=name", "Bearer synthetic-token"),
            (f"{metadata_url}/index.html?fields=name", "Bearer synthetic-token"),
            (f"{metadata_url}/jobs%2F?fields=name", "Bearer synthetic-token"),
        ])
        self.assertEqual(output, "Staged 6 objects in gs://app-bucket; 3 already present.\n")

    def test_new_only_asset_refusal_stops_all_html_uploads(self):
        upload_url = "https://example.invalid/upload"
        forbidden = HTTPError(upload_url, 403, "Forbidden", {}, io.BytesIO())
        policy_body = io.BytesIO(
            b'{"error":{"code":412,"message":"Request violates an OrgPolicy constraint.",'
            b'"errors":[{"domain":"global","reason":"orgPolicyConstraintFailed",'
            b'"message":"Request violates an OrgPolicy constraint."}]}}'
        )
        policy_refusal = HTTPError(upload_url, 412, "Precondition Failed", {}, policy_body)
        for refusal, expected_type, expected_message, expected_reads in (
            (forbidden, HTTPError, "HTTP Error 403: Forbidden", []),
            (policy_refusal, RuntimeError,
             "Cloud Storage refused to create 'assets/blocked.js' in bucket 'app-bucket': the object does not exist.",
             ["https://storage.googleapis.com/storage/v1/b/app-bucket/o/assets%2Fblocked.js?fields=name"]),
        ):
            with self.subTest(status=refusal.code):
                html_started = Event()
                requested = []
                reads = []
                missing_body = io.BytesIO()

                def open_url(request, timeout):
                    if request.get_method() == "GET":
                        reads.append(request.full_url)
                        raise HTTPError(request.full_url, 404, "Not Found", {}, missing_body)
                    name = request_name(request)
                    requested.append(name)
                    if name == "index.html":
                        html_started.set()
                    if name == "assets/blocked.js":
                        html_started.wait(0.5)
                        raise refusal
                    return io.BytesIO(b"{}")

                with self.assertRaises(expected_type) as raised:
                    self.stage_export(("index.html", "assets/allowed.js", "assets/blocked.js"), open_url)

                self.assertEqual(type(raised.exception), expected_type)
                self.assertEqual(str(raised.exception), expected_message)
                self.assertEqual(raised.exception is refusal, expected_type is HTTPError)
                self.assertCountEqual(requested, ["assets/allowed.js", "assets/blocked.js"])
                self.assertEqual(reads, expected_reads)
                self.assertEqual(refusal.fp.closed, bool(expected_reads))
                self.assertEqual(missing_body.closed, bool(expected_reads))


if __name__ == "__main__":
    unittest.main()
