# SPDX-License-Identifier: Apache-2.0
"""Drive a VM screen over VNC: check each screen by its text, then act on it."""

import json
import pathlib
import re
import subprocess
import time

from vnc import Vnc


class ScreenError(RuntimeError):
    pass


class Screen:
    """Each step waits for an expected text, then clicks, types, or presses keys."""

    def __init__(self, host, port, ocr, evidence, password=None):
        self.vnc = Vnc(host, port, password)
        self.ocr = pathlib.Path(ocr)
        self.evidence = pathlib.Path(evidence)
        self.evidence.mkdir(parents=True, exist_ok=True)
        self.count = 0
        self.items = []

    def read(self):
        self.vnc.refresh()
        self.count += 1
        path = self.evidence / f"screen-{self.count:04d}.png"
        self.vnc.png(path)
        output = subprocess.run([str(self.ocr), str(path)], capture_output=True,
                                text=True, check=True).stdout
        self.items = [json.loads(line) for line in output.splitlines() if line]
        if self.count > 3:
            (self.evidence / f"screen-{self.count - 3:04d}.png").unlink(missing_ok=True)
        return self.items

    def text(self):
        return " | ".join(item["text"] for item in self.items)

    def find(self, pattern):
        expression = re.compile(pattern, re.IGNORECASE)
        return [item for item in self.items if expression.search(item["text"])]

    def wait(self, pattern, timeout=120, absent=None):
        """Wait until a text matches. With absent, also wait until it is gone."""
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            self.read()
            if self.find(pattern) and not (absent and self.find(absent)):
                return
            time.sleep(2)
        self.fail(f"expected text {pattern!r} did not appear in {timeout} s")

    def wait_gone(self, pattern, timeout=120):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            self.read()
            if not self.find(pattern):
                return
            time.sleep(2)
        self.fail(f"text {pattern!r} stayed for {timeout} s")

    def click(self, pattern, last=True, below=None):
        """Click the text that matches. Buttons sit low, so the lowest match wins."""
        matches = self.find(pattern)
        if below is not None:
            matches = [item for item in matches if item["y"] > below]
        if not matches:
            self.fail(f"no control matches {pattern!r}")
        target = sorted(matches, key=lambda item: item["y"])[-1 if last else 0]
        self.vnc.click(target["x"], target["y"])
        time.sleep(1)

    def type(self, text):
        self.vnc.type(text)

    def press(self, name, modifiers=()):
        self.vnc.press(name, modifiers)
        time.sleep(0.3)

    def fail(self, message):
        path = self.evidence / "failure.png"
        self.vnc.png(path)
        raise ScreenError(f"{message}. Screen text: {self.text()}. Screenshot: {path}")

    def close(self):
        self.vnc.close()


def run_steps(screen, steps, log=print):
    """Run (name, expected pattern, actions, timeout) steps in order."""
    for name, expected, actions, timeout in steps:
        log(f"step: {name}")
        screen.wait(expected, timeout)
        for action, *values in actions:
            if action == "click":
                screen.click(*values)
            elif action == "type":
                screen.type(values[0])
            elif action == "press":
                screen.press(*values)
            elif action == "wait":
                screen.wait(*values)
            elif action == "gone":
                screen.wait_gone(*values)
            elif action == "sleep":
                time.sleep(values[0])
            else:
                raise ValueError(f"unknown action {action}")
