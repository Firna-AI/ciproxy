# SPDX-License-Identifier: Apache-2.0
"""Host commands and cleanup for the production-backed network smoke."""

import subprocess


def run(*args, input=None):
    result = subprocess.run(args, input=input, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(args[0] + " returned " + str(result.returncode) + ": " + result.stderr)
    return result.stdout


def running(unit):
    pid = subprocess.run(["systemctl", "show", "--property=MainPID", "--value", unit],
                         capture_output=True, text=True)
    return pid.returncode == 0 and pid.stdout.strip() not in ["", "0"]


def stop(unit):
    subprocess.run(["systemctl", "stop", unit], capture_output=True)
    return not running(unit)
