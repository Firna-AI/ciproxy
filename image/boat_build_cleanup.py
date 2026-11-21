# SPDX-License-Identifier: Apache-2.0
"""Preserve Boat's provider files during native upstream cleanup."""


PROTECTED_COMMANDS = {
    "rm -rf /tmp/*": "Boat keeps controller and provider temporary files.",
    "journalctl --vacuum-time=1s": "Boat keeps provider and system logs.",
    r'find /var/log -type f -regex ".*\.gz$" -delete': "Boat keeps provider and system logs.",
    r'find /var/log -type f -regex ".*\.[0-9]$" -delete': "Boat keeps provider and system logs.",
    r'find /var/log/ -type f -exec cp /dev/null {} \;': "Boat keeps provider and system logs.",
}


def adapt_cleanup(upstream):
    paths = ("images/ubuntu/scripts/build/cleanup.sh",
             "images/ubuntu/assets/post-gen/cleanup-logs.sh")
    changes = []
    for name in paths:
        path = upstream / name
        result, commands = [], []
        for line in path.read_text().splitlines(keepends=True):
            command = line.strip()
            if command in PROTECTED_COMMANDS:
                indent = line[:len(line) - len(line.lstrip())]
                ending = "\n" if line.endswith("\n") else ""
                # Keep every other upstream cleanup command and its original order.
                result.append(indent + ": # " + PROTECTED_COMMANDS[command] + ending)
                commands.append(command)
            else:
                result.append(line)
        path.write_text("".join(result))
        changes.append({"script": name, "omitted_cleanup_commands": commands,
                        "reason": "Preserve Boat temporary files and logs. Keep upstream cache, test-helper and apt-wrapper cleanup."})
    return changes
