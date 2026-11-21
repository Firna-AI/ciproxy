# SPDX-License-Identifier: Apache-2.0
"""Record provider directory owners and retain their upstream regression check."""

import json
import subprocess


DIRECTORIES = ("/usr", "/etc", "/usr/local", "/usr/local/bin")
RECORD_NAME = "provider-owners.json"
TEST_NAME = "<Directory> keeps its owner from before the build"


def capture(build_dir):
    path = build_dir / RECORD_NAME
    if path.exists():
        return
    owners = {directory: subprocess.check_output(["stat", "-c", "%U", directory], text=True).strip()
              for directory in DIRECTORIES}
    with path.open("x") as output:
        output.write(json.dumps(owners, indent=2) + "\n")


def adapt_owners(upstream):
    path = upstream / "images/ubuntu/scripts/tests/Tools.Tests.ps1"
    comment = "    # https://github.com/actions/runner-images/issues/14477\n"
    cases = "".join(f'        @{{ Directory = "{directory}" }}\n' for directory in DIRECTORIES)
    old = (comment + '    It "<Directory> is owned by root" -TestCases @(\n' + cases +
           '    ) {\n        $(stat -c "%U" $Directory) | Should -Be "root"\n    }\n')
    new = (comment + f'    It "{TEST_NAME}" -TestCases @(\n' + cases + '    ) {\n' +
           f'        $owners = Get-Content "/opt/ciproxy-build/{RECORD_NAME}" -Raw -ErrorAction Stop | '
           'ConvertFrom-Json -AsHashtable -ErrorAction Stop\n'
           '        if (-not $owners.ContainsKey($Directory)) {\n'
           '            throw "No owner was recorded before the build for $Directory"\n'
           '        }\n'
           '        $(stat -c "%U" $Directory) | Should -Be $owners[$Directory]\n'
           '    }\n')
    content = path.read_bytes()
    if content.count(old.encode()) != 1:
        raise RuntimeError("Pinned native adaptation does not match Tools.Tests.ps1")
    path.write_bytes(content.replace(old.encode(), new.encode(), 1))
    return {"test": "Tools.Tests.ps1: " + TEST_NAME,
            "reason": "Boat's base gives /usr/local and /usr/local/bin to its provider account; "
                      "the test checks that no build step changes the owner recorded before the build; "
                      "the machine image keeps the upstream root assertion."}
