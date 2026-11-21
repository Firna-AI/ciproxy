# SPDX-License-Identifier: Apache-2.0
"""Test the Boat profile and restore command without a provider request."""

import contextlib
import http.client
import io
import json
import os
import pathlib
import runpy
import shutil
import subprocess
import sys
import tempfile
import uuid
from unittest.mock import patch

from boat import Api


DEFINITION = pathlib.Path(__file__).resolve().parents[1]
FALLBACK = "/usr/local/lib/ciproxy/locales"
DEFAULT = "/usr/lib/locale"


def profile_paths(case, root, environment, home, **inherited):
    result = subprocess.run(["bash", "--noprofile", "--norc", "-ec", '''
for pass in 1 2; do
    for profile in "$1"/etc/profile.d/*.sh; do
        if [ -r "$profile" ]; then . "$profile"; fi
    done
    printf '%s\\n' "${LOCPATH-unset}"
done
''', "check-boat-profile", str(root)], env={**environment, "HOME": home, **inherited},
        capture_output=True, text=True)
    case.assertEqual((result.returncode, result.stderr), (0, ""))
    return result.stdout.splitlines()


def check_locale(case, workspace):
    if not all(shutil.which(tool) for tool in ("localedef", "locale")):
        case.skipTest("The Boat locale check needs localedef and locale.")
    i18n_path = os.environ.get("I18NPATH", "")
    sources = [pathlib.Path(path) for path in i18n_path.split(os.pathsep) if path]
    sources.append(pathlib.Path("/usr/share/i18n"))
    for name in ("locales/en_US", "charmaps/UTF-8"):
        if not any((source / name).is_file() or (source / (name + ".gz")).is_file()
                   for source in sources):
            case.skipTest(f"The Boat locale check needs glibc locale source {name}.")

    environment = {"PATH": os.environ["PATH"], "LC_ALL": "C"}
    with tempfile.TemporaryDirectory(dir=workspace) as temporary:
        root = pathlib.Path(temporary).resolve()
        (root / "etc/profile.d").mkdir(parents=True)
        (root / DEFAULT.lstrip("/")).mkdir(parents=True)
        install = subprocess.run(["bash", str(DEFINITION / "boat-build-locale.sh"), str(root)],
                                 env={**environment, "I18NPATH": i18n_path},
                                 capture_output=True, text=True)
        case.assertEqual(install.returncode, 0, install.stderr)

        provider = profile_paths(case, root, environment, "/home/user")
        paths = DEFAULT if provider[-1] == "unset" else provider[-1]
        fixture_paths = ":".join(str(root / path.lstrip("/")) for path in paths.split(":"))
        expected = (0, "UTF-8\n", "")

        def charmap(locale_name):
            command = subprocess.run(["bash", "--noprofile", "--norc", "-c", "locale charmap"],
                                     env={**environment, "LC_ALL": locale_name, "LOCPATH": fixture_paths},
                                     capture_output=True, text=True)
            return command.returncode, command.stdout, command.stderr

        for locale_name in ("en_US.UTF-8", "C.UTF-8"):
            with case.subTest(locale=locale_name):
                case.assertEqual(charmap(locale_name), expected)

        generated = root / FALLBACK.lstrip("/") / "en_US.utf8"
        case.assertTrue((generated / "LC_CTYPE").is_file())
        alias = f"ciproxy{uuid.uuid4().hex}"
        case.assertNotEqual(charmap(f"{alias}.UTF-8"), expected,
                            "The host must not supply the fixture locale alias.")
        generated.with_name(f"{alias}.utf8").symlink_to(generated.name, target_is_directory=True)
        case.assertEqual(charmap(f"{alias}.UTF-8"), expected)
        case.assertEqual(provider, [f"{FALLBACK}:{DEFAULT}"] * 2)
        case.assertEqual(profile_paths(case, root, environment, "/home/user", LOCPATH=f"/custom:{DEFAULT}"),
                         [f"{FALLBACK}:/custom:{DEFAULT}"] * 2)
        case.assertEqual(profile_paths(case, root, environment, "/home/runner"), ["unset", "unset"])


def check_locale_smoke(case):
    script = DEFINITION / "boat-locale-smoke.py"
    sandbox = "bx_fixture"
    passed = {"exitCode": 0, "stdout": "UTF-8\n", "stderr": "", "other": "private fixture data"}
    scenarios = (
        ("stderr", {**passed, "stderr": " \n"}, "Boat locale command did not have empty stderr."),
        ("stdout", {**passed, "stdout": "private fixture data"}, "Boat locale command did not report UTF-8."),
        ("exit", {**passed, "exitCode": 7}, "Boat locale command did not exit successfully."),
        ("request failure", http.client.BadStatusLine("private fixture data"), "Boat locale request failed."),
        ("success", passed, ""),
    )
    for scenario, reply, error in scenarios:
        with case.subTest(scenario=scenario):
            stdout, stderr = io.StringIO(), io.StringIO()
            api = Api.__new__(Api)
            response = {"return_value": reply} if isinstance(reply, dict) else {"side_effect": reply}
            with patch("boat.Api", return_value=api), patch.object(api, "command", **response) as command, \
                    patch.object(sys, "argv", [str(script), sandbox]), \
                    contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                with case.assertRaises(SystemExit) as exit_status:
                    runpy.run_path(str(script), run_name="__main__")
            case.assertEqual(exit_status.exception.code, 1 if error else 0)
            case.assertEqual(stderr.getvalue(), error + "\n" if error else "")
            case.assertEqual(stdout.getvalue(), "" if error else json.dumps({
                "codeset": "UTF-8", "exit_code": 0, "stderr_empty": True}) + "\n")
            command.assert_called_once()
            case.assertEqual(command.call_args.args[0], sandbox)
