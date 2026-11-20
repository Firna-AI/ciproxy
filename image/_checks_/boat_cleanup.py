# SPDX-License-Identifier: Apache-2.0
"""Execute adapted cleanup with safe command boundaries."""

import json
import os
import pathlib
import subprocess
import tempfile

from boat_build_cleanup import adapt_cleanup


LOG_CLEANUP = r'''# journalctl
if command -v journalctl; then
    journalctl --rotate
    journalctl --vacuum-time=1s
fi
find /var/log -type f -regex ".*\.gz$" -delete
find /var/log -type f -regex ".*\.[0-9]$" -delete
find /var/log/ -type f -exec cp /dev/null {} \;
'''


def check_cleanup(case, workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    upstream = root / "upstream"
    cleanup = upstream / "images/ubuntu/scripts/build/cleanup.sh"
    postgen = upstream / "images/ubuntu/assets/post-gen/cleanup-logs.sh"
    cleanup.parent.mkdir(parents=True)
    postgen.parent.mkdir(parents=True)
    cleanup.write_text("#!/bin/bash -e\napt-get clean\nrm -rf /tmp/*\nrm -rf /root/.cache\n" +
                       LOG_CLEANUP + 'rm -f /usr/local/bin/invoke_tests\nprefix=/usr/local/bin\n'
                       'for tool in apt apt-get apt-key; do sudo rm -f $prefix/$tool; done\n'
                       '# after cleanup\n')
    postgen.write_text("#!/bin/bash\n" + LOG_CLEANUP)
    tools = root / "tools"
    tools.mkdir()
    for name in ("apt-get", "rm", "find", "journalctl", "cp"):
        path = tools / name
        path.write_text("#!/usr/bin/env python3\nimport json, os, pathlib, sys\n"
                        "with open(os.environ['CLEANUP_EVENTS'], 'a') as output:\n"
                        "    output.write(json.dumps([pathlib.Path(sys.argv[0]).name, *sys.argv[1:]]) + '\\n')\n")
        path.chmod(0o755)
    sudo = tools / "sudo"
    sudo.write_text('#!/bin/sh\nexec "$@"\n')
    sudo.chmod(0o755)
    adapt_cleanup(upstream)
    expected = [
        ["apt-get", "clean"], ["rm", "-rf", "/root/.cache"], ["journalctl", "--rotate"],
        ["rm", "-f", "/usr/local/bin/invoke_tests"], ["rm", "-f", "/usr/local/bin/apt"],
        ["rm", "-f", "/usr/local/bin/apt-get"], ["rm", "-f", "/usr/local/bin/apt-key"],
    ]
    for script, commands in ((cleanup, expected), (postgen, [["journalctl", "--rotate"]])):
        events = root / (script.name + ".jsonl")
        environment = {**os.environ, "PATH": str(tools) + os.pathsep + os.environ["PATH"],
                       "CLEANUP_EVENTS": str(events)}
        result = subprocess.run(["bash", "-ef", str(script)], env=environment,
                                capture_output=True, text=True)
        case.assertEqual(result.returncode, 0, result.stderr)
        observed = [json.loads(line) for line in events.read_text().splitlines()]
        case.assertEqual(observed, commands, script.name)
