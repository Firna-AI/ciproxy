# SPDX-License-Identifier: Apache-2.0
"""The image variants of the macOS build.

full: the software of the GitHub image, with the pinned Xcode and all its
simulator runtimes.
standard: the same, with the iOS simulator runtime only and without the
Android SDK. It fits a Mac with 256 GB of storage.

Each change fails when the upstream text it changes is not there.
"""

import json

VARIANTS = ("full", "standard")
IOS_ONLY = [{"iOS": ["default"]}, {"watchOS": ["skip"]}, {"tvOS": ["skip"]},
            {"visionOS": ["skip"]}]
ANDROID_STEP = '      "${path.root}/../scripts/build/install-android-sdk.sh",\n'
ANDROID_TESTS = "scripts/tests/Android.Tests.ps1"
ANDROID_REPORT = '''# Android section
$android = $installedSoftware.AddHeader("Android")
$androidTable = Build-AndroidTable
$android.AddTable($androidTable)

$androidEnv = $android.AddHeader("Environment variables")
$androidEnv.AddTable($(Build-AndroidEnvironmentTable))

'''
REPORT = "scripts/docs-gen/Generate-SoftwareReport.ps1"


def check(variant):
    if variant not in VARIANTS:
        raise SystemExit(f"unknown image variant {variant!r}; use one of {VARIANTS}")


def toolset(text, variant):
    """The standard variant installs the iOS runtime of the default Xcode only."""
    check(variant)
    if variant == "full":
        return text
    data = json.loads(text)
    xcode = data["xcode"]
    for entry in xcode["arm64"]["versions"]:
        if entry["link"] == xcode["default"]:
            entry["install_runtimes"] = IOS_ONLY
    return json.dumps(data, indent=4) + "\n"


def template(text, variant):
    """The standard variant has no Android SDK step."""
    check(variant)
    if variant == "full":
        return text
    if text.count(ANDROID_STEP) != 1:
        raise SystemExit("the upstream template changed: the Android SDK step")
    return text.replace(ANDROID_STEP, "")


def tree(macos, variant):
    """The standard variant drops the Android tests and report section."""
    check(variant)
    if variant == "full":
        return
    tests = macos / ANDROID_TESTS
    report = macos / REPORT
    text = report.read_text()
    if not tests.is_file() or text.count(ANDROID_REPORT) != 1:
        raise SystemExit("the upstream tree changed: the Android tests or report section")
    tests.unlink()
    report.write_text(text.replace(ANDROID_REPORT, ""))
