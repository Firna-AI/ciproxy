"""The release image and the shared registry output use the registry host of the shared region."""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]


class RegistryHostTests(unittest.TestCase):
    def test_mirror_script_and_shared_output_use_the_shared_region_host(self):
        shared = (ROOT / "deploy/terraform/shared/main.tf").read_text()
        region = re.search(r'provider "google" \{[^}]*region\s*=\s*"([a-z0-9-]+)"', shared, re.S).group(1)
        host = f"{region}-docker.pkg.dev"
        output = re.search(r'runtime_repository\s*=\s*"([^/"]+)/', shared).group(1)
        self.assertEqual(output, host)
        script = (ROOT / "deploy/scripts/mirror-release-image.sh").read_text()
        hosts = set(re.findall(r"([a-z0-9-]+\.pkg\.dev)", script))
        self.assertEqual(hosts, {host})


if __name__ == "__main__":
    unittest.main()
