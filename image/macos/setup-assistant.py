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
        # OCR reads "Adult" badly. It is the row after "Child" and "Teen", and
        # a click on it leaves the screen.
        ("age", r"^Age Range$", [("row_after", r"^Child$", r"^Teen$")], 120),
        # The box is clear by default. The checkbox action clicks only a
        # checked box.
        ("location", r"^Enable Location Services$",
         [("uncheck", r"Location Services on this Mac"),
          ("click_until", CONTINUE, r"Don.t Use"), ("click", r"Don.t Use", False)], 180),
        ("time zone", r"Time Zone", [("click", CONTINUE)], 120),
        ("analytics", r"^Analytics$",
         [("uncheck", r"Share Mac Analytics"), ("click", CONTINUE)], 180),
        ("screen time", r"^Screen Time$", [("click", r"Set Up Later")], 120),
        # The dialog "Mac Data Will Not Be Securely Encrypted" has the only
        # Continue of this screen.
        ("filevault", r"Ready for FileVault",
         [("click_until", r"^Not Now$", r"Securely Encrypted"),
          ("click", CONTINUE, False)], 180),
        ("look", r"Choose Your Look", [("click", CONTINUE)], 120),
        ("updates", r"Update Mac Automatically",
         [("click", r"Only Download Automatically")], 120),
        ("welcome", r"Get Started|Welcome to Mac", [("click", r"Get Started|^Continue\.?$")], 600),
    ]


def resume_point(screen, password, timeout=900):
    """Wait for the first screen. Return True when the guest already shows
    the desktop, so a rerun after a failure past "welcome" skips the steps.
    On the login window, log in."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        screen.read()
        if screen.find(r"^Finder$"):
            return True
        if screen.find(r"Enter Password"):
            screen.type(password + "\n")
            time.sleep(10)
            continue
        if screen.find(r"^English"):
            return False
        time.sleep(3)
    screen.fail(f"neither the Setup Assistant nor the desktop appeared in {timeout} s")


def terminal(screen, password, public_key):
    """Open Terminal through Finder's Go menu, which needs no modifier key,
    and enable sudo and SSH for the build."""
    screen.wait(r"^Finder$", 600)
    screen.click(r"^Go$", False)
    screen.wait(r"^Utilities$", 30)
    screen.click(r"^Utilities$", False)
    screen.wait(r"^Terminal$", 60)
    # In icon view, the icon is above its label. In list view, the label is
    # the row, so the label is the second try.
    target = sorted(screen.find(r"^Terminal$"), key=lambda item: item["y"])[-1]
    prompt = rf"{USER}@|Last login"
    for y in (target["y"] - 2 * target["height"], target["y"]):
        screen.vnc.double_click(target["x"], y)
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline and not screen.find(prompt):
            time.sleep(2)
            screen.read()
        if screen.find(prompt):
            break
    screen.wait(prompt, 90)
    time.sleep(2)
    sudoers = f"{USER} ALL=(ALL) NOPASSWD: ALL"
    # The shell computes the marker, so the typed line does not match it.
    commands = [
        f"echo '{password}' | sudo -S sh -c \"echo '{sudoers}' > /etc/sudoers.d/{USER} "
        f"&& chmod 440 /etc/sudoers.d/{USER}\"",
        f"mkdir -p ~/.ssh && chmod 700 ~/.ssh && echo '{public_key}' >> ~/.ssh/authorized_keys",
        "chmod 600 ~/.ssh/authorized_keys",
        "sudo launchctl enable system/com.openssh.sshd",
        "sudo launchctl bootstrap system /System/Library/LaunchDaemons/ssh.plist; "
        "sudo launchctl print system/com.openssh.sshd >/dev/null && echo ciproxy-ssh-$((1+1))",
    ]
    for command in commands:
        screen.type(command + "\n")
        time.sleep(2)
    screen.wait(r"^ciproxy-ssh-2$", 60)


def main():
    work = pathlib.Path(sys.argv[1])
    status = json.loads((work / "run/status.json").read_text())
    secrets = work / "secrets"
    password = (secrets / "password").read_text().strip()
    screen = Screen("127.0.0.1", status["vnc_port"], work / "tools/ocr", work / "evidence/setup",
                    (secrets / "vnc-password").read_text().strip())
    try:
        if not resume_point(screen, password):
            run_steps(screen, steps(password))
        terminal(screen, password, (secrets / "id_ed25519.pub").read_text().strip())
    finally:
        screen.close()
    print("Setup Assistant done, SSH enabled")


if __name__ == "__main__":
    main()
