# SPDX-License-Identifier: Apache-2.0
"""A failed unit inspection must never authorize VM cleanup or a reset pass."""

import subprocess
import unittest
from unittest.mock import patch

from _smoke_.network_host import UnitState, stop, unit_state


class NetworkProbe(unittest.TestCase):
    def test_stop_requires_confirmed_absence_or_an_exited_process(self):
        for code, text, expected in [
            (1, "", UnitState.UNKNOWN),
            (0, "LoadState=not-found\n", UnitState.STOPPED),
            (0, "LoadState=loaded\nActiveState=active\nMainPID=123\n", UnitState.RUNNING),
            (0, "LoadState=loaded\nActiveState=deactivating\nMainPID=123\n", UnitState.RUNNING),
            (0, "LoadState=loaded\nActiveState=inactive\nMainPID=0\n", UnitState.STOPPED),
        ]:
            with self.subTest(expected=expected, text=text):
                results = [subprocess.CompletedProcess([], 1, "", "stop unavailable"),
                           subprocess.CompletedProcess([], code, text, "")]
                with patch("_smoke_.network_host.subprocess.run", side_effect=results):
                    self.assertEqual(stop("probe.service"), expected)
        with patch("_smoke_.network_host.subprocess.run", side_effect=OSError("bus unavailable")):
            self.assertEqual(stop("probe.service"), UnitState.UNKNOWN)
            self.assertEqual(unit_state("probe.service"), UnitState.UNKNOWN)
