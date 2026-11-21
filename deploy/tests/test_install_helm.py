"""Installer coverage without running downloaded Helm binaries."""

import io
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest


class HelmInstallerTests(unittest.TestCase):
    def setUp(self):
        self.repository = Path(__file__).resolve().parents[2]
        scratch = self.repository / ".context"
        scratch.mkdir(exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.runs = 0

    def command(self, directory, name, source):
        path = directory / name
        path.write_text(f"#!{sys.executable}\n{source}")
        path.chmod(0o755)

    def pinned_digest(self, system, machine):
        """Read the digest that the installer pins for a platform, or an unused value."""
        script = (self.repository / "deploy/scripts/install-helm.sh").read_text()
        for pattern, digest in re.findall(r"\n  ([^\n)]+)\)\n    platform=[^\n]+\n    checksum=(\w+)", script):
            if f"{system}/{machine}" in pattern.split("|"):
                return digest
        return "0" * 64

    def install(self, system, machine, checksum_tool="sha256sum", real_checksum=False):
        self.runs += 1
        directory = self.root / str(self.runs)
        commands = directory / "bin"
        commands.mkdir(parents=True)
        for name in ("mkdir", "tar", "gzip", "install"):
            (commands / name).symlink_to(shutil.which(name))
        self.command(commands, "uname", """import os, sys
print(os.environ['TEST_SYSTEM' if sys.argv[1] == '-s' else 'TEST_MACHINE'])
""")
        self.command(commands, "curl", """import json, os, pathlib, shutil, sys
url = next(argument for argument in sys.argv if argument.startswith('https://'))
with open(os.environ['DOWNLOAD_LOG'], 'a') as log:
    log.write(json.dumps(url) + '\\n')
shutil.copyfile(os.environ['FIXTURE_ARCHIVE'], sys.argv[sys.argv.index('--output') + 1])
""")
        if real_checksum:
            (commands / checksum_tool).symlink_to(shutil.which(checksum_tool))
        else:
            self.command(commands, checksum_tool, """import os, sys
with open(os.environ['CHECKSUM_LOG'], 'w') as log:
    log.write(' '.join(sys.argv[1:]) + '\\n')
print(os.environ.get('FAKE_DIGEST', '0' * 64) + '  ' + sys.argv[-1])
""")
        fixture = directory / "fixture.tar.gz"
        with tarfile.open(fixture, "w:gz") as archive:
            for platform in ("linux-amd64", "linux-arm64", "darwin-arm64", "darwin-amd64"):
                content = f"Inert Helm fixture: {platform}\n".encode()
                member = tarfile.TarInfo(f"{platform}/helm")
                member.size = len(content)
                archive.addfile(member, io.BytesIO(content))
        environment = {
            **os.environ,
            "PATH": str(commands),
            "TEST_SYSTEM": system,
            "TEST_MACHINE": machine,
            "FIXTURE_ARCHIVE": str(fixture),
            "DOWNLOAD_LOG": str(directory / "downloads.jsonl"),
            "CHECKSUM_LOG": str(directory / "checksum.log"),
            "FAKE_DIGEST": self.pinned_digest(system, machine),
            "RUNNER_TEMP": str(directory / "scratch"),
            "GITHUB_PATH": str(directory / "github-path"),
            "LC_ALL": "C",
            "TZ": "UTC",
        }
        result = subprocess.run(
            [shutil.which("bash"), "deploy/scripts/install-helm.sh"],
            cwd=self.repository, env=environment, capture_output=True, text=True,
        )
        return result, directory

    def test_installs_native_archive_and_keeps_ci_path_without_running_helm(self):
        cases = [
            ("Linux", "x86_64", "linux-amd64", "sha256sum"),
            ("Linux", "aarch64", "linux-arm64", "sha256sum"),
            ("Darwin", "arm64", "darwin-arm64", "shasum"),
            ("Darwin", "x86_64", "darwin-amd64", "shasum"),
        ]
        for system, machine, platform, checker in cases:
            with self.subTest(platform=platform):
                result, directory = self.install(system, machine, checker)
                self.assertEqual(result.returncode, 0, result.stderr)
                downloads = [json.loads(line) for line in
                             (directory / "downloads.jsonl").read_text().splitlines()]
                self.assertEqual(downloads, [
                    f"https://get.helm.sh/helm-v4.3.1-{platform}.tar.gz",
                ])
                scratch = directory / "scratch" / "ciproxy-helm"
                self.assertEqual(
                    (scratch / "helm").read_text(), f"Inert Helm fixture: {platform}\n",
                )
                self.assertEqual((directory / "github-path").read_text(), f"{scratch}\n")
                arguments = (directory / "checksum.log").read_text().splitlines()[0]
                # The installer hashes the archive and compares the digest itself.
                archive = str(scratch / f"helm-v4.3.1-{platform}.tar.gz")
                expected = archive if checker == "sha256sum" else f"-a 256 {archive}"
                self.assertEqual(arguments, expected)

    def test_bad_archive_checksum_stops_before_extraction_or_install(self):
        checker = "sha256sum" if shutil.which("sha256sum") else "shasum"
        result, directory = self.install("Linux", "x86_64", checker, real_checksum=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("SHA-256 verification failed", result.stderr)
        scratch = directory / "scratch" / "ciproxy-helm"
        self.assertFalse((scratch / "linux-amd64").exists())
        self.assertFalse((scratch / "helm").exists())
        self.assertFalse((directory / "github-path").exists())

    def test_unsupported_platform_stops_before_download(self):
        for system, machine in [("FreeBSD", "x86_64"), ("Linux", "ppc64le")]:
            with self.subTest(system=system, machine=machine):
                result, directory = self.install(system, machine)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(f"Unsupported Helm platform: {system}/{machine}", result.stderr)
                self.assertFalse((directory / "downloads.jsonl").exists())
                self.assertFalse((directory / "github-path").exists())


if __name__ == "__main__":
    unittest.main()
