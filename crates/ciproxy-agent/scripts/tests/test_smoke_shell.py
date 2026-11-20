"""Shell smoke entries keep cold-image work outside the job guards."""

from pathlib import Path
import sys
import unittest

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from smoke_image_support import index
from smoke_shell_support import ShellHost


class ShellReadinessTests(unittest.TestCase):
    def test_shell_entry_waits_before_warm_wait_and_dispatch(self):
        with ShellHost() as host:
            result = host.run('run_smoke warm\n')
            self.assertEqual(result.returncode, 19, result.stderr)
            self.assertEqual(host.observations(), ["published", "dispatch"])

    def test_fresh_host_waits_after_repair_before_first_dispatch(self):
        with ShellHost() as host:
            host.image.write_text(index("wanted", "wanted"))
            result = host.run(r'''
host_preflight() { host_evidence="$CIPROXY_SMOKE_EVIDENCE"; }
host_service_environment() { return 0; }
host_install() { printf 'install\n' >>"$SMOKE_EVENTS"; }
host_runtime_tools() { return 0; }
host_repair_probe() {
  printf 'repair\n' >>"$SMOKE_EVENTS"
  printf '{"newest":null,"available":[]}' >"$CIPROXY_SMOKE_IMAGE_INDEX"
}
host_main
''')
            self.assertEqual(result.returncode, 19, result.stderr)
            self.assertEqual(host.observations(), ["install", "repair", "published", "dispatch"])

    def test_shell_image_timeout_keeps_its_reason_and_never_dispatches(self):
        with ShellHost() as host:
            host.image.unlink()
            result = host.run('run_smoke host\n', STAY_COLD="1",
                              CIPROXY_SMOKE_IMAGE_TIMEOUT_SECONDS="7")
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertIn("image index is missing or unreadable", result.stderr)
            self.assertEqual(host.observations(), [])

    def test_failure_cancels_only_a_known_run_and_preserves_the_original_error(self):
        for run, cancel_exit, expected in [
            (None, "0", []),
            ("123", "0", ["run cancel 123 --repo owner/private-smoke"]),
            ("123", "23", ["run cancel 123 --repo owner/private-smoke"]),
        ]:
            with self.subTest(run=run, cancel_exit=cancel_exit), ShellHost() as host:
                setup = "" if run is None else "smoke_run=123\n"
                result = host.run(setup + "fail 'original smoke failure'\n", CANCEL_EXIT=cancel_exit)
                self.assertEqual(result.returncode, 1)
                self.assertEqual(result.stderr, "FAIL: original smoke failure\n")
                self.assertEqual(host.observations(), expected)


if __name__ == "__main__":
    unittest.main(verbosity=2)
