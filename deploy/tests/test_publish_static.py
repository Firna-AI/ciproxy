"""Regression coverage for static export aliases and object headers."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


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


if __name__ == "__main__":
    unittest.main()
