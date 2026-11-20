# SPDX-License-Identifier: Apache-2.0
"""Keep one preparation CLI and all other native build commands."""

import pathlib
import subprocess
import sys


def check_prepare_cli(case):
    image = pathlib.Path(__file__).resolve().parents[1]
    build = subprocess.run([sys.executable, str(image / "boat-build.py"), "--help"], capture_output=True, text=True)
    case.assertEqual(build.returncode, 0, build.stderr)
    case.assertNotIn("prepare", build.stdout)
    for action in ("bundle", "refresh", "report", "resume", "finalize"):
        case.assertIn(action, build.stdout)
    provider = subprocess.run([sys.executable, str(image / "boat.py"), "prepare", "--help"],
                              capture_output=True, text=True)
    case.assertEqual(provider.returncode, 0, provider.stderr)
    case.assertIn("bundle", provider.stdout)
