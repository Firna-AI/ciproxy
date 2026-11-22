#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Turn off System Integrity Protection in macOS Recovery over VNC.

GitHub's macOS VMs run without SIP, and the upstream recipe writes the
system TCC database. The guest must be started in Recovery. The steps open
Terminal from the Utilities menu, run csrutil disable with the `runner`
account, and turn the guest off.
"""

import json
import pathlib
import sys
import time

from screen_driver import Screen, run_steps

USER = "runner"


def steps(password):
    return [
        ("options", r"^Options$", [("click", r"^Options$"), ("click", r"^Continue$")], 600),
        ("user", rf"^{USER}$|Select a user", [("click", rf"^{USER}$"), ("click", r"^Next$")], 300),
        ("password", r"Enter Password|password", [("type", password + "\n")], 120),
        ("recovery", r"macOS Recovery|Restore from Time Machine|Reinstall macOS",
         [("click", r"^Utilities$", False), ("wait", r"^Terminal$", 30),
          ("click", r"^Terminal$")], 600),
    ]


def main():
    work = pathlib.Path(sys.argv[1])
    status = json.loads((work / "run/status.json").read_text())
    secrets = work / "secrets"
    password = (secrets / "password").read_text().strip()
    screen = Screen("127.0.0.1", status["vnc_port"], work / "tools/ocr",
                    work / "evidence/recovery", (secrets / "vnc-password").read_text().strip())
    try:
        run_steps(screen, steps(password))
        screen.wait(r"-bash|bash-|# ?$", 60)
        screen.type("csrutil disable\n")
        screen.wait(r"\[y/n\]|y/n", 60)
        screen.type("y\n")
        screen.wait(r"username", 60)
        screen.type(USER + "\n")
        screen.wait(r"[Pp]assword", 60)
        screen.type(password + "\n")
        screen.wait(r"System Integrity Protection is off|Restart the machine", 120)
        time.sleep(2)
        screen.type("shutdown -h now\n")
    finally:
        screen.close()
    print("SIP is off")


if __name__ == "__main__":
    main()
