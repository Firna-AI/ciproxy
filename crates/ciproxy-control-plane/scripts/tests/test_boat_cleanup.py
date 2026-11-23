"""Boat cleanup confirms its target and proves deletion before it succeeds."""

from contextlib import contextmanager
import io
import json
import os
from pathlib import Path
import sys
from tempfile import TemporaryDirectory
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


class PolicyTests(unittest.TestCase):
    @contextmanager
    def live(self, limit="4200", fail_restore=False):
        calls = []
        original = {
            "enabled": True,
            "wait_seconds": 60,
            "providers": [{"provider": "boat", "available": limit is not None,
                           "account": {"name": "Boat fixture"} if limit is not None else None,
                           "spend_limit_cents": limit}],
        }

        def api(_, method, route, body=None):
            calls.append((method, route, body))
            if method == "GET":
                return original
            if fail_restore and body == {"spend_limit_cents": limit}:
                raise ValueError("limit restore failed")
            return {"ok": True}

        def records():
            calls.append(("DB", "deliveries", None))
            return [{"delivery": "00000000-0000-0000-0000-000000000002"}]

        def intents():
            calls.append(("DB", "intents", None))
            return [{"runner": "00000000-0000-0000-0000-000000000003"}]

        with TemporaryDirectory() as directory:
            credentials = Path(directory) / "credentials.json"
            credentials.write_text(json.dumps({"base_url": "https://example.test",
                "workspace_id": "00000000-0000-0000-0000-000000000001",
                "workflow_id": "fixture.yml", "ref": "main"}))
            with patch.dict(os.environ, CIPROXY_OVERFLOW_CREDENTIALS_FILE=str(credentials)), \
                    patch.object(support.Live, "api", api), \
                    patch.object(support.Live, "records", side_effect=records), \
                    patch.object(support.Live, "intents", side_effect=intents):
                live = support.Live()
                self.addCleanup(live.state_file.unlink, missing_ok=True)
                yield live, calls

    def test_observe_only_command_does_not_read_overflow(self):
        with self.live() as (live, calls):
            self.assertEqual(live.restore(), [])
            self.assertEqual(calls, [])
            self.assertIsNone(json.loads(live.state_file.read_text())["original_policy"])

    def test_first_dispatch_saves_original_policy_before_dispatching(self):
        with self.live(limit=None) as (live, calls):
            def github(method, route, body=None):
                saved = json.loads(live.state_file.read_text())
                if method == "POST":
                    self.assertEqual(saved["original_policy"], {
                        "enabled": True, "wait_seconds": 60, "spend_limit_cents": None})
                    self.assertEqual(saved["markers"], [body["inputs"]["marker"]])
                    self.assertEqual(saved["delivery_baseline"], ["00000000-0000-0000-0000-000000000002"])
                    self.assertEqual(saved["intent_baseline"], ["00000000-0000-0000-0000-000000000003"])
                    return None
                return {"workflow_runs": [{"id": 42, "display_title": "ciproxy-smoke-" + saved["markers"][0]}]}

            with patch.object(live, "github", side_effect=github):
                self.assertEqual(live.dispatch("own", 1), 42)
            self.assertEqual(calls, [("GET", "/overflow", None), ("DB", "deliveries", None), ("DB", "intents", None)])

    def test_policy_uses_provider_limit_and_cleanup_restores_it_before_the_switch(self):
        limit_route = "/overflow/providers/boat/spend-limit"
        for fail_restore in [False, True]:
            with self.subTest(fail_restore=fail_restore), self.live(fail_restore=fail_restore) as (live, calls):
                live.policy("1", wait_seconds=30)
                live.policy("1000", wait_seconds=15)
                saved = json.loads(live.state_file.read_text())["original_policy"]
                self.assertEqual(saved, {"enabled": True, "wait_seconds": 60, "spend_limit_cents": "4200"})
                self.assertEqual(live.restore(), ["settings restoration"] if fail_restore else [])
                expected = [
                    ("GET", "/overflow", None),
                    ("DB", "deliveries", None),
                    ("DB", "intents", None),
                    ("PUT", limit_route, {"spend_limit_cents": "1"}),
                    ("PUT", "/overflow", {"enabled": True, "wait_seconds": 30}),
                    ("PUT", limit_route, {"spend_limit_cents": "1000"}),
                    ("PUT", "/overflow", {"enabled": True, "wait_seconds": 15}),
                    ("PUT", "/overflow", {"enabled": False, "wait_seconds": 60}),
                    ("PUT", limit_route, {"spend_limit_cents": "4200"}),
                ]
                if not fail_restore:
                    expected.append(("PUT", "/overflow", {"enabled": True, "wait_seconds": 60}))
                self.assertEqual(calls, expected)


if __name__ == "__main__":
    unittest.main(verbosity=2)
