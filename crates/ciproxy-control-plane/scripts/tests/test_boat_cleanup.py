"""Boat cleanup confirms its target and proves deletion before it succeeds."""

from contextlib import contextmanager
import io
import json
from pathlib import Path
import sys
import unittest
from urllib.error import HTTPError
from unittest.mock import patch

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from overflow_smoke import support


def operation(sandbox, status="completed", identity="delete-1", **changes):
    return {"ok": True, "type": "deletion.operation", "operation": {
        "id": identity, "kind": "sandbox", "targetId": sandbox, "status": status, **changes}}


def info(sandbox, state="archived"):
    return {"ok": True, "type": "sandbox.info", "sandbox": {"id": sandbox, "state": state}}


class Boat:
    def __init__(self, responses):
        self.responses = iter(responses)
        self.requests = []
        self.now = 0

    def open(self, request, timeout):
        self.requests.append((request, timeout))
        response = next(self.responses)
        if response == 404:
            raise HTTPError(request.full_url, 404, "missing", {}, io.BytesIO())
        return io.BytesIO(json.dumps(response).encode())

    def sleep(self, seconds):
        self.now += seconds

    @contextmanager
    def live(self, deadline=6):
        live = support.Live.__new__(support.Live)
        live.config = {"boat_api_key": "", "boat_wallet": "personal"}
        live.opener = self
        live.cleanup_deadline = deadline
        with patch.object(support.time, "monotonic", lambda: self.now), \
                patch.object(support.time, "sleep", self.sleep):
            yield live


class CleanupTests(unittest.TestCase):
    def assert_confirmation(self, boat, sandbox):
        for request, _ in boat.requests:
            self.assertEqual(request.full_url, "https://boat.dev/api/v1/sandboxes/" + sandbox)
            self.assertEqual(request.get_header("X-boat-org"), "personal")
            self.assertEqual(request.get_header("X-ascii-confirm-delete"),
                             sandbox if request.method == "DELETE" else None)

    def test_destroy_confirms_each_target_and_waits_for_terminal_observation(self):
        for sandbox, ended in [("bx_23456789", info("bx_23456789")), ("bx_abcdefgh", 404)]:
            with self.subTest(sandbox=sandbox):
                boat = Boat([operation(sandbox, "processing"), operation(sandbox),
                             info(sandbox, "running"), ended])
                with boat.live() as live:
                    live.destroy(sandbox)
                self.assertEqual([request.method for request, _ in boat.requests],
                                 ["DELETE", "DELETE", "GET", "GET"])
                self.assertEqual([timeout for _, timeout in boat.requests], [6, 5, 5, 4])
                self.assert_confirmation(boat, sandbox)

    def test_delete_404_still_needs_absence_and_a_running_sandbox_reaches_deadline(self):
        sandbox = "bx_23456789"
        for responses, expected, methods in [
            ([404, 404, 404], None, ["DELETE", "GET", "GET"]),
            ([404, info(sandbox, "running"), 404, info(sandbox), info(sandbox)], None,
             ["DELETE", "GET", "DELETE", "GET", "GET"]),
            ([operation(sandbox), info(sandbox, "running"), info(sandbox, "running")],
             "observation deadline reached", ["DELETE", "GET", "GET"]),
        ]:
            with self.subTest(expected=expected, methods=methods):
                boat = Boat(responses)
                error = None
                with boat.live(deadline=2) as live:
                    try:
                        live.destroy(sandbox)
                    except ValueError as failure:
                        error = str(failure)
                self.assertEqual(error, expected)
                self.assertEqual([request.method for request, _ in boat.requests], methods)
                self.assert_confirmation(boat, sandbox)

    def test_cleanup_keeps_identity_and_operation_status_checks(self):
        sandbox = "bx_23456789"
        for responses, expected in [
            ([operation("bx_abcdefgh")], "deletion identity mismatch"),
            ([operation(sandbox, "processing"), operation(sandbox, identity="delete-2")],
             "deletion operation changed"),
            ([operation(sandbox, "unrecognized")], "unknown deletion status"),
            ([operation(sandbox), info("bx_abcdefgh")], "sandbox identity mismatch"),
        ]:
            with self.subTest(expected=expected):
                boat = Boat(responses)
                with boat.live() as live:
                    with self.assertRaisesRegex(ValueError, "^" + expected + "$"):
                        live.destroy(sandbox)
                self.assert_confirmation(boat, sandbox)
        boat = Boat([404])
        with boat.live() as live:
            with self.assertRaisesRegex(ValueError, "^request failed$"):
                live.boat("DELETE", sandbox)
        self.assert_confirmation(boat, sandbox)


if __name__ == "__main__":
    unittest.main(verbosity=2)
