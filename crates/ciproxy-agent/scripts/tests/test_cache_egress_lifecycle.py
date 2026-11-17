"""Cache inspection works before budget replay and still requires real reports."""

from pathlib import Path
import sys
import unittest

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import cache_egress_host as host
import cache_egress_lifecycle as lifecycle
from cache_lifecycle_support import CacheHost


def outcome(action):
    try:
        action()
    except host.Failure as error:
        return str(error)
    return None


class InstalledCacheTests(unittest.TestCase):
    def test_readiness_preserves_unknown_budget_and_requires_authentication(self):
        state = CacheHost()
        with state.installed() as cache:
            self.assertIsNone(outcome(cache.inspect), "Readiness required an unavailable usage report")
            self.assertIsNone(state.budget)
            self.assertEqual([(request.method, request.full_url, request.get_header("Authorization"))
                              for request, _, _ in state.requests], [
                ("GET", "http://127.0.0.1:8229/", "Bearer public-admin-probe"),
                ("GET", "http://127.0.0.1:8229/", None),
            ])
        for options in [{"authorized": False}, {"denies_anonymous": False}]:
            with self.subTest(options=options):
                with CacheHost(**options).installed() as cache:
                    self.assertEqual(outcome(cache.inspect),
                                     "Cache administration readiness did not succeed")

    def test_lifecycle_waits_for_real_report_after_resume_and_keeps_report_failures(self):
        for replay, expected, report_count, clock in [
            ("ready", None, 3, 2),
            ("unauthorized", "Cache administration did not succeed", 1, 0),
            ("never", "The cache usage report did not become available", 60, 60),
        ]:
            with self.subTest(replay=replay):
                state = CacheHost(replay=replay)
                with state.installed() as cache:
                    error = outcome(lambda: lifecycle.lifecycle(cache))
                self.assertEqual(error, expected)
                self.assertEqual(state.reports, report_count)
                self.assertEqual(state.now, clock)
                requests = [(request, resumed, budget) for request, resumed, budget
                            in state.requests if request.full_url.endswith("/report")]
                self.assertEqual([resumed for _, resumed, _ in requests], [True] * report_count)
                self.assertEqual([request.method for request, _, _ in state.requests
                                  if request.full_url.endswith("/budget")], [])
                self.assertEqual(state.budget, 1234 if replay == "ready" else None)
                self.assertEqual(state.files["/etc/ciproxy-cache.nft"], b"public valid policy\n")
                self.assertEqual(state.agent_active, True)
                self.assertEqual([name for name in state.files if "firewall-probe-" in name], [])


if __name__ == "__main__":
    unittest.main(verbosity=2)
