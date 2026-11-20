# SPDX-License-Identifier: Apache-2.0
"""Run native Ubuntu source preparation on provider and generic URIs."""

import pathlib
import subprocess
import tempfile


def check_sources(case, workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    sources = root / "ubuntu.sources"
    sources.write_text('''Types: deb
URIs: mirror+file:/etc/apt/boat-mirrors.txt
Suites: noble noble-updates noble-backports

Types: deb
URIs: https://archive.ubuntu.com/ubuntu/
Suites: noble-security

Types: deb
URIs: http://security.ubuntu.com/ubuntu
Suites: noble-security
''')
    command = pathlib.Path(__file__).resolve().parents[1] / "boat-build-sources.py"
    result = subprocess.run(["python3", str(command), str(sources)], capture_output=True, text=True)
    case.assertEqual(result.returncode, 0, result.stdout + result.stderr)
    case.assertEqual(sources.read_text(), '''Types: deb
URIs: http://azure.archive.ubuntu.com/ubuntu/
Suites: noble noble-updates noble-backports

Types: deb
URIs: http://azure.archive.ubuntu.com/ubuntu/
Suites: noble-security

Types: deb
URIs: http://security.ubuntu.com/ubuntu
Suites: noble-security
''')
