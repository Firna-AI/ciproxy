#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Complete the Setup Assistant of a new guest over VNC, then enable SSH.

Each step waits for the text of its screen, then clicks a control by its
label or types. An unexpected screen stops the build with a screenshot.
The account is `runner`, as on GitHub. The steps skip the Apple Account,
location, analytics, Screen Time, and FileVault. Then Terminal adds a
sudoers file without a password, the build's SSH key, and Remote Login.
"""

import json
import pathlib
import sys
import time

from screen_driver import Screen, run_steps

USER = "runner"
# OCR can read the button with a full stop.
CONTINUE = r"^Continue\.?$"
AGREE = r"^Agree\.?$"


def steps(password):
    return [
        # "English" is the US entry. "English (UK)" sets the British input
        # source and lists the United Kingdom first.
        ("language", r"^English \(UK\)$|^English$",
         [("select", r"^English$", r"^English", "English"), ("press", "return")], 900),
        ("country", r"Country or Region",
         [("select", r"^United States$", r"^(Afghanistan|Albania|Algeria)$", "United States"),
          ("click", CONTINUE)], 120),
        ("transfer", r"Transfer Your Data|Migration Assistant",
         [("click", r"Set up as new|Not Now"), ("click", CONTINUE)], 120),
        ("languages", r"Written and Spoken Languages", [("click", CONTINUE)], 120),
        ("accessibility", r"^Accessibility$", [("click", r"^Not Now$")], 120),
        ("privacy", r"Data & Privacy", [("click", CONTINUE)], 120),
        # Creating the account takes about 40 s.
        ("account", r"Create a Mac Account",
         [("click", r"^Full Name$", False), ("type", USER), ("press", "tab"),
          ("press", "tab"), ("type", password), ("press", "tab"), ("type", password),
          ("uncheck", r"Allow computer account password to be reset"),
          ("click", CONTINUE)], 180),
        # A mouse click on the menu item does not register, but the keys do.
        ("apple account", r"Sign In to Your Apple Account",
         [("click", r"Other Sign-In Options"), ("sleep", 1), ("press", "down"),
          ("press", "down"), ("press", "return"), ("sleep", 2), ("click", CONTINUE),
          ("wait", r"^Skip$", 60), ("click", r"^Skip$")], 300),
        # A dialog repeats the page's buttons over the greyed page. The dialog
        # sits above the page's bottom buttons, so its button is the topmost.
        ("terms", r"Terms and Conditions",
         [("click", AGREE), ("wait", r"I ?have read and agree", 30),
          ("click", AGREE, False)], 180),
        ("age", r"Age Range|age range", [("click", r"^Adult"), ("click", CONTINUE)], 120),
        ("location", r"Location Services",
         [("uncheck", r"Location Services on this Mac"), ("click", CONTINUE),
          ("wait", r"Don.t Use", 30), ("click", r"Don.t Use", False)], 180),
        ("time zone", r"Time Zone", [("click", CONTINUE)], 120),
        ("analytics", r"^Analytics$",
         [("uncheck", r"Share Mac Analytics"), ("click", CONTINUE)], 180),
        ("screen time", r"^Screen Time$", [("click", r"Set Up Later")], 120),
        ("filevault", r"FileVault",
         [("click", r"^Not Now$"), ("sleep", 2), ("click", CONTINUE, False)], 180),
        ("look", r"Choose Your Look", [("click", CONTINUE)], 120),
        ("updates", r"Update Mac Automatically|Software Update|Keep Your Mac Up to Date",
         [("click", r"Only Download Automatically"), ("click", CONTINUE)], 120),
        ("welcome", r"Get Started|Welcome to Mac", [("click", r"Get Started|^Continue\.?$")], 600),
    ]


def terminal(screen, password, public_key):
    """Open Terminal from Spotlight and enable sudo and SSH for the build."""
    screen.wait(r"^Finder$", 600)
    screen.press("space", ("command",))
    time.sleep(2)
    screen.type("Terminal")
    time.sleep(2)
    screen.press("return")
    screen.wait(rf"{USER}@|Last login|\$ ?$|% ?$", 120)
    sudoers = f"{USER} ALL=(ALL) NOPASSWD: ALL"
    commands = [
        f"echo '{password}' | sudo -S sh -c \"echo '{sudoers}' > /etc/sudoers.d/{USER} "
        f"&& chmod 440 /etc/sudoers.d/{USER}\"",
        f"mkdir -p ~/.ssh && chmod 700 ~/.ssh && echo '{public_key}' >> ~/.ssh/authorized_keys",
        "chmod 600 ~/.ssh/authorized_keys",
        "sudo launchctl enable system/com.openssh.sshd",
        "sudo launchctl bootstrap system /System/Library/LaunchDaemons/ssh.plist; "
        "echo ciproxy-ssh-ready",
    ]
    for command in commands:
        screen.type(command + "\n")
        time.sleep(2)
    screen.wait(r"ciproxy-ssh-ready", 60)


def main():
    work = pathlib.Path(sys.argv[1])
    status = json.loads((work / "run/status.json").read_text())
    secrets = work / "secrets"
    password = (secrets / "password").read_text().strip()
    screen = Screen("127.0.0.1", status["vnc_port"], work / "tools/ocr", work / "evidence/setup",
                    (secrets / "vnc-password").read_text().strip())
    try:
        run_steps(screen, steps(password))
        terminal(screen, password, (secrets / "id_ed25519.pub").read_text().strip())
    finally:
        screen.close()
    print("Setup Assistant done, SSH enabled")


if __name__ == "__main__":
    main()
