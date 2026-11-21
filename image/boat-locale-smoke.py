#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check the controller locale of a restored Boat sandbox before launch."""

import argparse
import http.client
import json
import sys

from boat import Api


COMMAND = ("python3 -I -S -c 'import locale; locale.setlocale(locale.LC_ALL, \"\"); "
           "print(locale.nl_langinfo(locale.CODESET))'")


def fail(message):
    print(message, file=sys.stderr)
    return 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("sandbox")
    args = parser.parse_args()
    try:
        answer = Api().command(args.sandbox, COMMAND)
    except (http.client.HTTPException, KeyError, OSError, ValueError):
        return fail("Boat locale request failed.")

    exit_code = answer.get("exitCode") if isinstance(answer, dict) else None
    if type(exit_code) is not int or exit_code != 0:
        return fail("Boat locale command did not exit successfully.")
    if answer.get("stderr") != "":
        return fail("Boat locale command did not have empty stderr.")
    if answer.get("stdout") not in ("UTF-8", "UTF-8\n"):
        return fail("Boat locale command did not report UTF-8.")
    print(json.dumps({"codeset": "UTF-8", "exit_code": 0, "stderr_empty": True}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
