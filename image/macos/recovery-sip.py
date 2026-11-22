#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Turn off System Integrity Protection in macOS Recovery over VNC.

GitHub's macOS VMs run without SIP, and the upstream recipe writes the
system TCC database. The guest must be started in Recovery. The steps open
Options from the startup picker, choose the language, open Terminal from the
Utilities menu, run csrutil disable as `runner`, and turn the guest off.
"""

import json
import pathlib
import sys
import time

from screen_driver import Screen, run_steps

USER = "runner"


def steps():
    return [
        # The tile's label is not the control. A click on the gear icon above
        # it selects the tile, and Continue appears under it.
        ("options", r"^Options$",
         [("double_click_above", r"^Options$", 70), ("wait", r"^Continue\.?$", 30),
          ("click", r"^Continue\.?$")], 300),
    ]


RECOVERY_WINDOW = r"Reinstall macOS|Disk Utility|Restore from Time Machine"


def reach_recovery(screen, password, timeout=900):
    """Wait for the Recovery window. On the way, answer each optional screen
    once: the language list, a user picker, and a password field. macOS
    26.6.2 shows only the language list, but FileVault adds a login."""
    handled = set()
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        time.sleep(3)
        screen.read()
        if screen.find(RECOVERY_WINDOW):
            return
        if "language" not in handled and screen.find(r"^English \(UK\)$"):
            screen.click(r"^English$", False)
            screen.press("return")
            handled.add("language")
        elif "user" not in handled and screen.find(r"Select a user"):
            screen.click(rf"^{USER}$")
            screen.click(r"^Next$")
            handled.add("user")
        elif "password" not in handled and screen.find(r"Enter.*[Pp]assword"):
            screen.type(password + "\n")
            handled.add("password")
    screen.fail(f"the Recovery window did not appear in {timeout} s")


def open_terminal(screen):
    """A click on the menu item does nothing, but the keys work. Terminal is
    the third item of the Utilities menu."""
    # OCR merges the menu bar, such as "Utilities Window", so the click
    # goes to the word. The Command key does not reach the guest over VNC.
    screen.click_word("Utilities")
    screen.wait(r"Startup Security Utility", 30)
    for _ in range(3):
        screen.press("down")
    screen.press("return")
    screen.wait(r"-bash|bash-\d|# ?$", 120)
    time.sleep(2)


def disable_sip(screen, password, timeout=180):
    """Answer the prompts of csrutil disable in the order that they come."""
    screen.type("csrutil disable\n")
    answers = [(r"\[y/n\]|\(y/n\)", "y"), (r"Authorized user|[Uu]sername", USER),
               (r"[Pp]assword", password)]
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
        run_steps(screen, steps())
        reach_recovery(screen, password)
        open_terminal(screen)
        disable_sip(screen, password)
        time.sleep(2)
        screen.type("shutdown -h now\n")
    finally:
        screen.close()
    print("SIP is off")


if __name__ == "__main__":
    main()
