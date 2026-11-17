#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check Boat delete confirmation without a provider credential or request."""

import contextlib
import io
import json
import os
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True

import boat


class DeleteConfirmation(unittest.TestCase):
    def test_delete_confirms_the_target_sandbox(self):
        for sandbox in ["bx_23456789", "bx_abcdefgh"]:
            with self.subTest(sandbox=sandbox):
                response = io.BytesIO(b'{"ok":true}')
                output = io.StringIO()
                with patch.dict(os.environ, {
                    "BOAT_API_KEY": "fixture-key",
                    "BOAT_API_BASE": "https://boat.example/api/v1",
                }), patch.object(sys, "argv", ["boat.py", "delete", sandbox]), \
                        patch("urllib.request.OpenerDirector.open", return_value=response) as opened, \
                        contextlib.redirect_stdout(output):
                    self.assertEqual(boat.main(), 0)
                opened.assert_called_once()
                request = opened.call_args.args[0]
                self.assertEqual(request.method, "DELETE")
                self.assertEqual(request.full_url,
                                 "https://boat.example/api/v1/sandboxes/" + sandbox)
                self.assertEqual(request.get_header("X-ascii-confirm-delete"), sandbox)
                self.assertEqual(json.loads(output.getvalue()), {
                    "sandbox_id": sandbox, "delete_requested": True,
                })


if __name__ == "__main__":
    unittest.main(verbosity=2)
