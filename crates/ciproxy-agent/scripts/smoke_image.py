"""Wait for the verified image selected for new job VMs."""

import json
import os
from pathlib import Path
import sys
import time


class ImageReadinessError(Exception):
    """The selected image did not become ready."""


def wait_for_image():
    path = Path(os.environ.get("CIPROXY_SMOKE_IMAGE_INDEX", "/var/lib/ciproxy/area/images/index.json"))
    expected = os.environ.get("CIPROXY_SMOKE_IMAGE_RELEASE")
    # An 18 GB archive takes 96 minutes at 25 Mbit/s. Allow verification and extraction too.
    timeout = int(os.environ.get("CIPROXY_SMOKE_IMAGE_TIMEOUT_SECONDS", "7200"))
    deadline = time.monotonic() + timeout
    target = "the selected job image" if expected is None else "job image release " + expected
    print(f"WAIT: {target}; limit {timeout} seconds", flush=True)
    while True:
        selected, reason = selected_image(path)
        if selected is not None:
            if expected is None or selected == expected:
                print("READY: job image release " + selected, flush=True)
                return selected
            reason = f"selected release is {selected}; expected {expected}"
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise ImageReadinessError(f"Timed out after {timeout} seconds waiting for {target}: {reason}")
        time.sleep(min(5, remaining))


def selected_image(path):
    try:
        report = json.loads(path.read_text())
    except OSError:
        return None, "image index is missing or unreadable"
    except ValueError:
        return None, "image index is malformed"
    if not isinstance(report, dict) or not isinstance(report.get("available"), list):
        return None, "image index is malformed"
    newest = report.get("newest")
    if newest is None:
        return None, "no image release is selected"
    if not isinstance(newest, str) or any(
            not isinstance(image, dict) or not isinstance(image.get("image_id"), str)
            for image in report["available"]):
        return None, "image index is malformed"
    if any(image["image_id"] == newest for image in report["available"]):
        return newest, None
    return None, "newest does not name an available image"


def main():
    try:
        wait_for_image()
        return 0
    except (ImageReadinessError, ValueError) as error:
        print("FAIL: " + str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
