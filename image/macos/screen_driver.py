# SPDX-License-Identifier: Apache-2.0
"""Drive a VM screen over VNC: check each screen by its text, then act on it."""

import json
import pathlib
import re
import subprocess
import time

from vnc import Vnc, VncError


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
        try:
            self.vnc.refresh()
        except (OSError, VncError) as error:
            self.fail(f"VNC error: {error!r}")
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

    def select(self, pattern, row, text, timeout=30):
        """Select a list entry. When it is not on screen, click any row and
        type its name, so the list jumps to it."""
        if not self.find(pattern):
            self.click(row, False)
            self.vnc.type(text)
            time.sleep(2)
            self.wait(pattern, timeout)
        self.click(pattern, False)

    def uncheck(self, pattern):
        """Clear the checkbox at the left edge of a label. A checked box is
        filled with the blue accent colour, so the build looks for that
        colour from the label's first character leftward."""
        matches = self.find(pattern)
        if not matches:
            self.fail(f"no checkbox label matches {pattern!r}")
        label = matches[0]
        left = label["x"] - label["width"] // 2
        y = label["y"]
        box = self.blue_box(left + label["height"], y)
        if box is None:
            return
        self.vnc.click(box, y)
        time.sleep(1)
        self.read()
        if self.blue_box(left + label["height"], y) is not None:
            self.fail(f"the checkbox of {pattern!r} stayed checked")

    def blue_box(self, start, y):
        """The x of the first accent-blue pixel left of start, or None."""
        self.vnc.refresh()
        for x in range(int(start), max(int(start) - 60, 0), -2):
            blue, _green, red = self.vnc.pixel(x, y)
            if blue > 150 and red < 120 and blue - red > 80:
                return x - 4
        return None

    def click_until(self, pattern, expected, timeout=60, every=15):
        """Click, then wait for the expected text, such as a dialog. Click
        again while the control stays and the text does not come."""
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if self.find(pattern):
                self.click(pattern)
            retry = min(time.monotonic() + every, deadline)
            while time.monotonic() < retry:
                time.sleep(2)
                self.read()
                if self.find(expected):
                    return
        self.fail(f"{expected!r} did not appear after clicks on {pattern!r}")

    def row_after(self, first, second):
        """Click the row after two rows, at the same spacing. For a row whose
        text OCR reads badly, such as "Adult" after "Child" and "Teen"."""
        above, below = self.find(first), self.find(second)
        if not above or not below:
            self.fail(f"no rows match {first!r} and {second!r}")
        top, middle = above[0], below[0]
        self.vnc.click(middle["x"], 2 * middle["y"] - top["y"])
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


def run_action(screen, action, values):
    if action == "click":
        screen.click(*values)
    elif action == "uncheck":
        screen.uncheck(*values)
    elif action == "click_until":
        screen.click_until(*values)
    elif action == "row_after":
        screen.row_after(*values)
    elif action == "select":
        screen.select(*values)
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


def leave(screen, name, expected, last, timeout, retry_after=30):
    """Wait until the step's screen text is gone. While it stays, click the
    last control again, because a click can come before the control is
    ready. A slow step, such as the account creation, can take a minute."""
    deadline = time.monotonic() + timeout
    retry = time.monotonic() + retry_after
    while time.monotonic() < deadline:
        screen.read()
        if not screen.find(expected):
            return
        if last[0] in ("click", "press", "row_after") and time.monotonic() >= retry:
            if last[0] != "click" or screen.find(last[1]):
                run_action(screen, last[0], last[1:])
            retry = time.monotonic() + retry_after
        time.sleep(2)
    screen.fail(f"step {name}: the screen {expected!r} stayed for {timeout} s")


def run_steps(screen, steps, log=print):
    """Run (name, expected pattern, actions, timeout) steps in order. Each
    step ends when its screen text is gone."""
    for name, expected, actions, timeout in steps:
        log(f"step: {name}")
        try:
            screen.wait(expected, timeout)
            for action, *values in actions:
                run_action(screen, action, values)
            leave(screen, name, expected, actions[-1], max(timeout, 120))
        except (OSError, VncError) as error:
            screen.fail(f"step {name}: VNC error: {error!r}")
