# SPDX-License-Identifier: Apache-2.0
"""Report installed package differences from the build's APT preferences."""

import fnmatch
import json
import pathlib


def stanzas(content):
    for paragraph in content.replace("\r\n", "\n").split("\n\n"):
        fields = {}
        for line in paragraph.splitlines():
            key, separator, value = line.partition(":")
            if separator and not line.startswith((" ", "\t", "#")):
                fields[key] = value.strip()
        if fields:
            yield fields


def compare_pins(root):
    root = pathlib.Path(root)
    result = {"differences_fail_build": False, "difference_count": 0, "differences": []}
    try:
        pins = json.loads(pathlib.Path(__file__).with_name("software-apt-pins.json").read_text())["packages"]
        installed = list(stanzas((root / "var/lib/dpkg/status").read_text()))
    except (OSError, UnicodeError, json.JSONDecodeError, KeyError, TypeError):
        return {**result, "available": False, "reason": "APT pin or installed-package metadata is unavailable."}
    for package in installed:
        if package.get("Status", "").split()[-1:] != ["installed"]:
            continue
        name, version = package.get("Package", ""), package.get("Version", "")
        source = (package.get("Source") or name).split(" ", 1)[0]
        expected = []
        for names, pin in pins.items():
            patterns = names.split()
            if any(fnmatch.fnmatchcase(source, pattern[4:]) if pattern.startswith("src:") else
                   fnmatch.fnmatchcase(name, pattern) for pattern in patterns):
                expected.append(pin)
        if expected and not any(fnmatch.fnmatchcase(version, pin) for pin in expected):
            result["differences"].append({"package": name, "installed": version, "pin": " | ".join(sorted(set(expected)))})
    result["differences"].sort(key=lambda item: item["package"])
    result.update(available=True, difference_count=len(result["differences"]))
    return result
