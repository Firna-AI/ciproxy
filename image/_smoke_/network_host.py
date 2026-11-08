# SPDX-License-Identifier: Apache-2.0
"""Host commands and cleanup for the production-backed network smoke."""

from enum import Enum
import subprocess


class UnitState(Enum):
    RUNNING = "running"
    STOPPED = "stopped"
    UNKNOWN = "unknown"


def run(*args, input=None):
    result = subprocess.run(args, input=input, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(args[0] + " returned " + str(result.returncode) + ": " + result.stderr)
    return result.stdout


def unit_state(unit):
    try:
        result = subprocess.run(
            ["systemctl", "show", "--property=LoadState,ActiveState,MainPID", unit],
            capture_output=True, text=True)
    except OSError:
        return UnitState.UNKNOWN
    values = dict(line.split("=", 1) for line in result.stdout.splitlines() if "=" in line)
    if values.get("LoadState") == "not-found":
        return UnitState.STOPPED
    if result.returncode != 0:
        return UnitState.UNKNOWN
    if values.get("ActiveState") in ["active", "activating", "deactivating", "reloading"]:
        return UnitState.RUNNING
    if (values.get("LoadState") == "loaded"
            and values.get("ActiveState") in ["inactive", "failed"]
            and values.get("MainPID") == "0"):
        return UnitState.STOPPED
    return UnitState.UNKNOWN


def stop(unit):
    try:
        subprocess.run(["systemctl", "stop", unit], capture_output=True)
    except OSError:
        pass
    return unit_state(unit)
