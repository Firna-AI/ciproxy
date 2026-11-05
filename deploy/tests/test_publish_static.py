"""Regression coverage for static export aliases and object headers."""

import io
import json
from email import policy
from email.parser import BytesParser
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from deploy.scripts import publish_static


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
                    self.assertEqual(parts[1].get_payload(decode=True), b"offline-payload")
                    self.assertEqual(request.get_method(), "POST")
                    self.assertEqual(request.full_url, "https://storage.googleapis.com/upload/storage/v1/b/private-static-origin/o?uploadType=multipart")


if __name__ == "__main__":
    unittest.main()
