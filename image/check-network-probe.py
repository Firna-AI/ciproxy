#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check network probe lifecycle handling without systemd or KVM."""

import sys
import unittest

sys.dont_write_bytecode = True

from _checks_.network_probe import NetworkProbe


if __name__ == "__main__":
    unittest.main(verbosity=2)
