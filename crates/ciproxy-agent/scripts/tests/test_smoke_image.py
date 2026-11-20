"""Image readiness starts before the job clocks and the first workflow."""

from contextlib import redirect_stdout
import io
from pathlib import Path
import sys
import unittest

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import own_smoke
import smoke_image
from smoke_image_support import ImageHost, StopAtDispatch, index


def outcome(action):
    try:
        return action()
    except (own_smoke.Failure, smoke_image.ImageReadinessError) as error:
        return str(error)


class ImageReadinessTests(unittest.TestCase):
    def test_waits_for_matching_newest_and_expected_release(self):
        host = ImageHost(FileNotFoundError(), "{", index(None, "wanted"),
                         index("missing", "wanted"), index("old", "old", "wanted"),
                         index("wanted", "old", "wanted"))
        with host.installed(CIPROXY_SMOKE_IMAGE_RELEASE="wanted"), redirect_stdout(io.StringIO()):
            selected = outcome(smoke_image.wait_for_image)
        self.assertEqual(selected, "wanted")
        self.assertEqual(host.events, ["published"])
        self.assertGreater(host.now, 0)

    def test_ready_selected_image_needs_no_expected_release(self):
        host = ImageHost(index("selected", "older", "selected"))
        with host.installed(), redirect_stdout(io.StringIO()):
            selected = outcome(smoke_image.wait_for_image)
        self.assertEqual(selected, "selected")
        self.assertEqual(host.now, 0)

    def test_image_timeout_has_its_own_default_and_override(self):
        for limit, environment, source, reason in [
            (7200, {}, FileNotFoundError(), "image index is missing or unreadable"),
            (7, {"CIPROXY_SMOKE_IMAGE_TIMEOUT_SECONDS": "7"}, "{", "image index is malformed"),
        ]:
            with self.subTest(limit=limit):
                host = ImageHost(source)
                with host.installed(**environment), redirect_stdout(io.StringIO()):
                    error = outcome(own_smoke.Smoke)
                self.assertEqual(error, f"Timed out after {limit} seconds waiting for the selected job image: {reason}")
                self.assertEqual(host.now, limit)
                self.assertEqual(host.events, [])

    def test_python_entry_waits_before_warm_wait_and_dispatch(self):
        host = ImageHost(index(None), index("wanted", "wanted"))
        with host.installed(), redirect_stdout(io.StringIO()):
            smoke = own_smoke.Smoke()
            smoke.wait(smoke.warm, "warm readiness")
            with self.assertRaises(StopAtDispatch):
                smoke.dispatch("warm")
        self.assertEqual(host.events, ["published", "warm", "dispatch"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
