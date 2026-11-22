#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Turn off System Integrity Protection in macOS Recovery over VNC.

GitHub's macOS VMs run without SIP, and the upstream recipe writes the
system TCC database. The guest must be started in Recovery. The steps open
Options from the startup picker, log in as `runner`, open Terminal from the
Utilities menu, run csrutil disable, and turn the guest off.
"""

import json
import pathlib
import sys
import time

from screen_driver import Screen, run_steps

USER = "runner"


def steps(password):
    return [
        # The tile's label is not the control. A click on the gear icon above
        # it selects the tile, and Continue appears under it.
        ("options", r"^Options$",
         [("double_click_above", r"^Options$", 70), ("wait", r"^Continue\.?$", 30),
          ("click", r"^Continue\.?$")], 300),
        ("user", r"Select a user",
         [("click", rf"^{USER}$"), ("click", r"^Next$")], 900),
        ("password", r"Enter.*[Pp]assword|^Password$",
         [("type", password + "\n")], 300),
    ]


def open_terminal(screen):
    """The Recovery window stays behind Terminal, so this is no step."""
    screen.wait(r"macOS Recovery|Reinstall macOS|Disk Utility|Safari", 900)
    screen.click(r"^Utilities$", False)
    screen.wait(r"^Terminal$", 30)
    screen.click(r"^Terminal$", False)
    screen.wait(r"-bash|bash-\d|# ?$", 90)
    time.sleep(2)


def disable_sip(screen, password, timeout=180):
    """Answer the prompts of csrutil disable in the order that they come."""
    screen.type("csrutil disable\n")
    answers = [(r"\[y/n\]|\(y/n\)", "y"), (r"[Uu]sername", USER), (r"[Pp]assword", password)]
    answered = set()
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        time.sleep(3)
        screen.read()
        if screen.find(r"System Integrity Protection is off|Restart the machine"):
            return
        for prompt, answer in answers:
            if prompt not in answered and screen.find(prompt):
                screen.type(answer + "\n")
                answered.add(prompt)
                break
    screen.fail(f"csrutil disable did not finish in {timeout} s")


def main():
    work = pathlib.Path(sys.argv[1])
    status = json.loads((work / "run/status.json").read_text())
    secrets = work / "secrets"
    password = (secrets / "password").read_text().strip()
    screen = Screen("127.0.0.1", status["vnc_port"], work / "tools/ocr",
                    work / "evidence/recovery", (secrets / "vnc-password").read_text().strip())
    try:
        run_steps(screen, steps(password))
        open_terminal(screen)
        disable_sip(screen, password)
        time.sleep(2)
        screen.type("shutdown -h now\n")
    finally:
        screen.close()
    print("SIP is off")


if __name__ == "__main__":
    main()
