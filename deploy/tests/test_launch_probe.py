"""Operator launch proofs require unexpired tokens and actual refusal."""

from datetime import datetime, timezone
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from unittest.mock import patch
from urllib.error import HTTPError

from deploy.scripts import launch_probe


class Response(io.BytesIO):
    status = 200

    def __init__(self, document):
        super().__init__(json.dumps(document).encode())


class LaunchProbeTests(unittest.TestCase):
    def setUp(self):
        scratch = Path(__file__).resolve().parents[2] / ".context/launch-probe-tests"
        scratch.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name) / "tokens"
        self.mode = "active"

    def http(self, call, **_):
        mint = call.full_url.endswith(":generateAccessToken")
        if "iam.googleapis.com/v1/projects" in call.full_url:
            return Response({"disabled": self.mode != "enabled"})
        allowed = self.mode == "active" or self.mode == ("fresh-allowed" if mint else "old-allowed")
        if allowed:
            return Response({"accessToken": "private-captured-token", "expireTime": "2026-11-05T10:00:00Z"} if mint else {})
        raise HTTPError(call.full_url, 403, "refused", {}, None)

    def invoke(self, mode):
        output, error = io.StringIO(), io.StringIO()
        with (patch.object(launch_probe.subprocess, "check_output", return_value="private-operator-token\n"),
              patch.object(launch_probe, "urlopen", side_effect=self.http),
              patch.object(launch_probe, "utc_now", return_value=datetime(2026, 11, 5, 9, tzinfo=timezone.utc)),
              redirect_stdout(output), redirect_stderr(error)):
            status = launch_probe.main([mode, str(self.root)])
        self.assertNotIn("private-captured-token", output.getvalue() + error.getvalue())
        self.assertNotIn("private-operator-token", output.getvalue() + error.getvalue())
        return status, output.getvalue()

    def test_capture_baseline_and_disabled_account_negative_proof(self):
        self.assertEqual(self.invoke("capture")[0], 0)
        self.assertEqual(os.stat(self.root / "shared.json").st_mode & 0o777, 0o600)
        self.mode = "revoked"
        status, output = self.invoke("verify")
        self.assertEqual(status, 0)
        self.assertEqual(output.count("unexpired token refused"), 2)

    def test_expired_tokens_and_incomplete_revocation_cannot_pass(self):
        self.assertEqual(self.invoke("capture")[0], 0)
        for mode in ("old-allowed", "fresh-allowed", "enabled"):
            with self.subTest(mode=mode):
                self.mode = mode
                self.assertEqual(self.invoke("verify")[0], 1)
        self.mode = "revoked"
        path = self.root / "shared.json"
        document = json.loads(path.read_text())
        document["expireTime"] = "2026-11-05T08:59:59Z"
        path.write_text(json.dumps(document))
        self.assertEqual(self.invoke("verify")[0], 1)
