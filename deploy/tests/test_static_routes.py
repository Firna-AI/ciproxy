"""Route fresh exported objects through the actual mocked Terraform plan."""

import json
from pathlib import Path
import re
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[2]


class StaticRouteTests(unittest.TestCase):
    def test_export_inventory_selects_immutable_mutable_html_and_reserved_backends(self):
        for export in ("site", "app"):
            self.assertTrue((ROOT / "ts" / export / "dist/index.html").is_file(),
                            f"Build {export} before running the deployment inventory test.")
        init = subprocess.run(["terraform", "-chdir=deploy/terraform/edge", "init", "-backend=false",
                               "-input=false", "-lockfile=readonly"], cwd=ROOT, text=True, capture_output=True)
        self.assertEqual(init.returncode, 0, init.stdout + init.stderr)
        run = subprocess.run(["terraform", "-chdir=deploy/terraform/edge", "test", "-json", "-verbose",
                              "-filter=cache.tftest.hcl"], cwd=ROOT, text=True, capture_output=True)
        self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
        plans = [event["test_plan"] for line in run.stdout.splitlines()
                 if (event := json.loads(line)).get("type") == "test_plan"]
        self.assertEqual(len(plans), 1)
        resources = {item["address"]: item["change"]["after"] for item in plans[0]["resource_changes"]}
        mapping = resources["google_compute_url_map.https"]
        backends = {item["id"]: item for address, item in resources.items()
                    if address.startswith("google_compute_backend_bucket.")}
        counts = {"immutable": 0, "mutable": 0, "html": 0, "reserved": 0}
        for export, host in (("site", "ciproxy.dev"), ("app", "app.ciproxy.dev")):
            public = ROOT / "ts" / export / "public"
            public_names = {str(path.relative_to(public)) for path in public.rglob("*") if path.is_file()}
            result = subprocess.run(["python3", "deploy/scripts/publish_static.py", "--directory=ts/" + export + "/dist",
                                     "--bucket=dry-run", "--dry-run"], cwd=ROOT, text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            objects = json.loads(result.stdout)
            for item in [*objects, {"name": "", "contentType": "text/html; charset=utf-8", "cacheControl": "no-store"}]:
                path = "/" + item["name"]
                # Independent filename rules, never the routing prefixes.
                # Pagefind pf_meta shares its parent with stable files and
                # deliberately receives the safe short lifetime too.
                html = item["contentType"].startswith("text/html")
                hashed = item["name"] not in public_names and bool(re.search(r"\.[A-Za-z0-9_-]{6,16}\.(?:js|css|woff2?)$", path)
                              or re.search(r"/ec\.[a-z0-9]{5}\.(?:js|css)$", path)
                              or re.search(r"[.-][a-f0-9]{32}(?:@[234]x)?\.[a-z0-9]+$", path)
                              or re.search(r"/[a-z]{2}_[a-f0-9]{7,}\.pf_(?:index|fragment)$", path))
                expected = "html" if html else "immutable" if hashed else "mutable"
                with self.subTest(export=export, path=path, expected=expected):
                    service, rewritten = select(mapping, host, path)
                    backend = backends[service]
                    policy = backend["cdn_policy"][0]
                    origin = "site" if export == "site" else "dashboard"
                    suffix = "html" if html else "assets" if hashed else "mutable"
                    self.assertEqual(service, "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendBuckets/" + origin + "-" + suffix,
                                     f"{expected} objects must use their own {origin} backend")
                    self.assertEqual(backend["bucket_name"], "ciproxy-production-" + origin + "-ciproxy")
                    if html:
                        self.assertEqual(policy["cache_mode"], "USE_ORIGIN_HEADERS")
                        self.assertEqual(item["cacheControl"], "no-store")
                        self.assertEqual(rewritten, "/index.html" if path == "/" else path)
                    else:
                        self.assertEqual(policy["cache_mode"], "FORCE_CACHE_ALL")
                        self.assertEqual(policy["default_ttl"], 31536000 if hashed else 300)
                        self.assertEqual(policy["max_ttl"], 31536000 if hashed else 300)
                        self.assertEqual(policy["client_ttl"], 86400 if hashed else 300)
                    counts[expected] += 1
        for path, suffix in (("/api", "backendServices/control"), ("/api/health/ready", "backendServices/control"),
                             ("/agent/connect", "backendServices/control"), ("/hooks/github", "backendServices/control"),
                             ("/images", "backendBuckets/signed-images"), ("/images/workspace/image", "backendBuckets/signed-images")):
            with self.subTest(path=path):
                self.assertEqual(select(mapping, "app.ciproxy.dev", path)[0],
                                 "https://www.googleapis.com/compute/v1/projects/ciproxy/global/" + suffix)
                counts["reserved"] += 1
        self.assertEqual(select(mapping, "get.ciproxy.dev", "/ciproxy-agent")[0],
                         "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendServices/release")
        self.assertTrue(all(count > 0 for count in counts.values()), counts)


def select(mapping, host, path):
    """Google path rules choose exact paths, then the longest terminal /*."""
    name = next(rule["path_matcher"] for rule in mapping["host_rule"] if host in rule["hosts"])
    matcher = next(item for item in mapping["path_matcher"] if item["name"] == name)
    exact = [rule for rule in matcher["path_rule"] if path in rule["paths"]]
    prefixes = [(len(pattern), rule) for rule in matcher["path_rule"] for pattern in rule["paths"]
                if pattern.endswith("/*") and path.startswith(pattern[:-1])]
    rule = exact[0] if exact else max(prefixes, key=lambda pair: pair[0])[1] if prefixes else None
    if rule is None:
        return matcher["default_service"], path
    actions = rule.get("route_action") or []
    rewrites = actions[0].get("url_rewrite") if actions else []
    return rule["service"], rewrites[0]["path_prefix_rewrite"] if rewrites else path
