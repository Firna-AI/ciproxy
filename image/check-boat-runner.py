#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check the Boat runner profile and stderr without a real registration."""

import base64
import binascii
import json
import os
import pathlib
import select
import shutil
import subprocess
import sys
import tempfile
import unittest

from _checks_.boat_fixture import copy_program, copy_python


IMAGE = pathlib.Path(__file__).resolve().parent
WORKSPACE = pathlib.Path(".context/image-checks/boat-runner")
JIT_VARIABLE = "ACTIONS_RUNNER_INPUT_JITCONFIG"
SECRET = "fixture-value"
NOTICE = b"[ciproxy] runner stderr truncated\n"
PROFILE_ERROR = b"[ciproxy] runner entry error: nvm.sh:3 status=1\n"
PROFILE = '''
export NVM_DIR="$HOME/.nvm"
__ascii_nvm_path() {
    [ -d "$NVM_DIR/selected/bin" ] || return 1
    printf '%s' "$NVM_DIR/selected/bin"
}
__ascii_nvm_bin="$(__ascii_nvm_path 2>/dev/null)"
if [ -n "$__ascii_nvm_bin" ]; then
    export PATH="$__ascii_nvm_bin:$PATH"
    nvm() { printf 'lazy nvm\\n'; }
elif [ -s "$NVM_DIR/nvm.sh" ]; then
    . "$NVM_DIR/nvm.sh"
fi
'''


def write_script(path, content):
    path.write_text(content)
    path.chmod(0o755)


def fixture_jit():
    credential = base64.b64encode(json.dumps({"data": {"key": SECRET}}).encode()).decode()
    return base64.b64encode(json.dumps({".credentials": credential}).encode()).decode()


def recording_journal(root):
    write_script(root / "usr/bin/systemd-cat", '''#!/bin/bash
test -z "${ACTIONS_RUNNER_INPUT_JITCONFIG+x}" || exit 33
printf '%s\\n' "$@" > /observations/identifier
exec /usr/bin/cat > /observations/journal
''')
    return root / "observations/journal"


def write_failing_profile(root, diagnostics=""):
    (root / "etc/profile").write_text(diagnostics + ". /etc/profile.d/nvm.sh\n")
    (root / "etc/profile.d").mkdir()
    (root / "etc/profile.d/nvm.sh").write_text('''test -z "${BASH_ENV+x}"
startup_failure() {
    test "$ACTIONS_RUNNER_INPUT_JITCONFIG" = mismatch
}
startup_failure
''')


def prepare_root(root):
    assert os.geteuid() == 0, "Run the fixed-path fixture with sudo"
    root.chmod(0o755)
    for folder in ("dev", "etc", "opt/actions-runner/bin", "observations",
                   "home/runner", "usr/local/libexec", "usr/bin"):
        (root / folder).mkdir(parents=True)
    for name in ("bash", "cat", "cp", "env"):
        copy_program(shutil.which(name), root, f"/usr/bin/{name}")
    copy_python(root)
    if getattr(binascii, "__file__", None):
        copy_program(binascii.__file__, root)
    (root / "bin").symlink_to("usr/bin")
    (root / "dev/null").touch(mode=0o666)
    (root / "dev/null").chmod(0o666)
    for source, target, mode in (
        ("boat-runner-entry.sh", "ciproxy-runner", 0o755),
        ("runner-entry.sh", "ciproxy-runner-common", 0o755),
        ("boat-runner-stderr.py", "ciproxy-boat-runner-stderr.py", 0o644),
        ("boat-runner-errors.sh", "ciproxy-boat-runner-errors.sh", 0o644),
    ):
        destination = root / "usr/local/libexec" / target
        shutil.copyfile(IMAGE / source, destination)
        destination.chmod(mode)
    os.chown(root / "observations", 12345, 12345)
    os.chown(root / "opt/actions-runner/bin", 12345, 12345)


def in_runner_root(root, *command, registration="fixture-jit", source=None,
                   stderr=subprocess.PIPE, pass_fds=(), timeout=None):
    return subprocess.run(
        [shutil.which("chroot"), "--userspec=12345:12345", str(root), *command],
        env={"PATH": "/usr/bin:/bin", "HOME": "/home/runner", "USER": "runner",
             "LOGNAME": "runner", JIT_VARIABLE: registration},
        stdin=source, stdout=subprocess.PIPE, stderr=stderr, pass_fds=pass_fds, timeout=timeout,
    )


class BoatRunner(unittest.TestCase):
    def setUp(self):
        WORKSPACE.mkdir(parents=True, exist_ok=True)
        directory = tempfile.TemporaryDirectory(dir=WORKSPACE)
        self.addCleanup(directory.cleanup)
        self.root = pathlib.Path(directory.name).resolve()

    def test_optional_nvm_path_keeps_system_node_and_both_nvm_branches(self):
        profile = self.root / "nvm.sh"
        profile.write_text(PROFILE)
        nvm = self.root / ".nvm"
        nvm.mkdir()
        environment = {"HOME": str(self.root), "PATH": "/usr/bin:/bin"}

        def load_profile():
            return subprocess.run(
                ["bash", "-ec", '. "$1"; printf "continued:%s\\n" "$PATH"; '
                 'if declare -F nvm >/dev/null; then nvm; fi', "boat-profile", str(profile)],
                env=environment, capture_output=True, text=True,
            )

        def guard():
            return subprocess.run([sys.executable, str(IMAGE / "boat-build-nvm.py"), str(profile)],
                                  capture_output=True, text=True, check=True)

        self.assertEqual(load_profile().returncode, 1, "Fixture must show the old errexit failure")
        guard()
        result = load_profile()
        self.assertEqual((result.returncode, result.stdout, result.stderr),
                         (0, "continued:/usr/bin:/bin\n", ""))
        (nvm / "nvm.sh").write_text("printf 'eager nvm\\n'\n")
        self.assertEqual(load_profile().stdout, "eager nvm\ncontinued:/usr/bin:/bin\n")
        selected = nvm / "selected/bin"
        selected.mkdir(parents=True)
        self.assertEqual(load_profile().stdout, f"continued:{selected}:/usr/bin:/bin\nlazy nvm\n")
        guarded = profile.read_bytes()
        result = guard()
        self.assertEqual((result.stdout, result.stderr), ("", ""))
        self.assertEqual(profile.read_bytes(), guarded)
        profile.unlink()
        self.assertIn("missing", guard().stderr)
        self.assertFalse(profile.exists())
        profile.write_text("false\n")
        self.assertIn("no recognized lookup", guard().stderr)
        self.assertEqual(profile.read_text(), "false\n")
        self.assertEqual(load_profile().returncode, 1)

    def test_stderr_retains_a_redacted_prefix_and_drains_the_burst(self):
        prepare_root(self.root)
        journal = recording_journal(self.root)
        jit = fixture_jit()
        short_jit = base64.b64encode(b"{}").decode()
        opaque = b"aZ47_/b+" * 8
        prefix = (PROFILE_ERROR + b"/usr/local/libexec/ciproxy-runner-common: line 19: "
                  b"ACTIONS_RUNNER_INPUT_JITCONFIG: JIT registration required\n")
        urls = (b"Database connection failed: postgresql://demo:fixture@db/app\n"
                b"ICU is missing; see https://example.test/guide then ftp://example.test/faq\n"
                b"Redirect: example://local?token=fake\n"
                b"Redirect: example://local/" + SECRET.encode()
                + b"\nRedirect: example://" + opaque + b"\n")
        burst = (prefix + urls + jit.encode() + b"\nvalue: " + SECRET.encode()
                 + b"\nAuthorization: Bearer abc\nCookie: short\npassword=fake\n" + opaque
                 + b"\n" + b"next diagnostic\n" * 8192 + b"late diagnostic\n")
        wide_line = (prefix + short_jit.encode() + b"\n"
                     + b"z" * 131072 + b"\nlate diagnostic\n")
        safe_prefix = PROFILE_ERROR + b"[ciproxy] runner entry error: JIT registration required\n"
        safe_urls = (b"Database connection failed: postgresql://[redacted]\n"
                     b"ICU is missing; see https://[redacted]\n"
                     b"[redacted]\n[redacted]\n[redacted]\n")
        for registration, data, expected_prefix in (
            (jit, burst, safe_prefix + safe_urls), (short_jit, wide_line, safe_prefix),
        ):
            with self.subTest(bytes=len(data)):
                journal.unlink(missing_ok=True)
                with tempfile.TemporaryFile(dir=self.root) as source:
                    source.write(data)
                    source.seek(0)
                    result = in_runner_root(self.root, "/usr/bin/python3", "-I",
                                            "/usr/local/libexec/ciproxy-boat-runner-stderr.py",
                                            registration=registration, source=source)
                    self.assertEqual(result.returncode, 0, result.stderr.decode())
                    self.assertEqual(source.tell(), len(data), "Logger must drain all stderr")
                self.assertEqual((result.stdout, result.stderr), (b"", b""))
                retained = journal.read_bytes()
                self.assertIn(b"--identifier=ciproxy-boat-runner-stderr\n",
                              (self.root / "observations/identifier").read_bytes())
                self.assertEqual(retained[:len(expected_prefix)], expected_prefix)
                self.assertTrue(retained.endswith(NOTICE))
                self.assertEqual(retained.count(NOTICE), 1)
                self.assertLessEqual(len(retained), 16 * 1024 + len(NOTICE))
                excluded = {"encoded JIT": registration.encode(), "decoded value": SECRET.encode(),
                            "bearer": b"Bearer abc", "cookie": b"Cookie: short", "password": b"password=fake",
                            "opaque value": opaque, "output beyond the cap": b"late diagnostic",
                            "oversized line": b"zzzzzzzz"}
                for description, value in excluded.items():
                    self.assertFalse(value in retained, f"Journal must exclude {description}")

    def test_entry_redirects_before_profiles_and_survives_a_journal_failure(self):
        prepare_root(self.root)
        write_script(self.root / "usr/bin/systemd-cat", '''#!/bin/bash
printf 'journal failure\\n'
printf 'journal failure\\n' >&2
exit 37
''')
        (self.root / "etc/profile").write_text('''
/usr/bin/python3 -I -c 'import os, stat; assert stat.S_ISFIFO(os.fstat(2).st_mode)'
printf 'profile stderr\\n' >&2
''')
        (self.root / "etc/environment").touch()
        runner = self.root / "opt/actions-runner/bin"
        (runner / "Runner.Worker.stock.dll").write_bytes(b"stock")
        write_script(runner / "Runner.Listener", '''#!/bin/bash
set -e
test -z "${BASH_ENV+x}"
/usr/bin/cat /observations/burst >&2
printf 'Listening for Jobs\\n%s\\n' "$*"
exit 23
''')
        (self.root / "observations/burst").write_bytes(b"listener diagnostic\n" * 8192)
        with tempfile.TemporaryFile(dir=self.root) as stderr:
            result = in_runner_root(self.root, "/usr/local/libexec/ciproxy-runner",
                                    source=subprocess.DEVNULL, stderr=stderr)
            stderr.seek(0)
            self.assertEqual((result.returncode, result.stdout, stderr.read()),
                             (23, b"Listening for Jobs\nrun --once\n", b""))

    def test_silent_profile_error_keeps_status_and_reports_only_source_location(self):
        prepare_root(self.root)
        write_failing_profile(self.root)
        result = in_runner_root(
            self.root, "/usr/bin/env", "BASH_ENV=/usr/local/libexec/ciproxy-boat-runner-errors.sh",
            "/usr/local/libexec/ciproxy-runner-common", source=subprocess.DEVNULL,
        )
        self.assertEqual((result.returncode, result.stdout, result.stderr), (1, b"", PROFILE_ERROR))

    def test_entry_writes_redacted_jit_and_profile_error_to_the_journal(self):
        prepare_root(self.root)
        journal = recording_journal(self.root)
        write_failing_profile(self.root, f"printf '%s\\n' 'value: {SECRET}' >&2\n")

        read_end, write_end = os.pipe()
        with os.fdopen(read_end, "rb") as completion:
            try:
                result = in_runner_root(
                    self.root, "/usr/local/libexec/ciproxy-runner", registration=fixture_jit(),
                    source=subprocess.DEVNULL, pass_fds=(write_end,), timeout=30,
                )
            finally:
                os.close(write_end)
            self.assertTrue(select.select([completion], [], [], 30)[0],
                            "Runner stderr helper did not finish")
            self.assertEqual(completion.read(1), b"", "Completion pipe must reach EOF")

        self.assertEqual((result.returncode, result.stdout, result.stderr), (1, b"", b""))
        self.assertTrue(journal.exists(), "Runner stderr did not reach the recording writer")
        self.assertEqual(journal.read_bytes(), b"[redacted]\n" + PROFILE_ERROR)


if __name__ == "__main__":
    unittest.main(verbosity=2)
