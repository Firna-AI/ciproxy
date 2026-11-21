#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Drain runner stderr and forward only bounded, redacted diagnostics."""

import base64
import json
import os
import queue
import re
import subprocess
import sys
import threading


JIT_VARIABLE = "ACTIONS_RUNNER_INPUT_JITCONFIG"
LOG_LIMIT = 16 * 1024
LINE_LIMIT = 4096
NOTICE = b"[ciproxy] runner stderr truncated\n"
REDACTED = b"[redacted]\n"
PRIVATE = re.compile(
    r"authoriz|bearer|credential|token|cookie|password|passwd|secret|signature|"
    r"(?:api|private)[ _-]?key|jit[ _-]?config", re.IGNORECASE,
)
PRIVATE_URL = re.compile(r"https?://(?:[^/\s]*@|[^\s]*\?)", re.IGNORECASE)
OPAQUE = re.compile(r"[A-Za-z0-9+/_=-]{32,}")


def decode(value):
    try:
        padded = value + "=" * (-len(value) % 4)
        return base64.b64decode(padded, altchars=b"-_", validate=True).decode()
    except ValueError:
        return ""


def read_json(value):
    try:
        return json.loads(value)
    except (ValueError, RecursionError):
        return None


def private_values(document, private=False):
    pending = [(document, private)]
    while pending:
        value, private = pending.pop()
        if isinstance(value, dict):
            pending.extend((child, private or bool(PRIVATE.search(name)))
                           for name, child in value.items())
        elif isinstance(value, list):
            pending.extend((child, private) for child in value)
        elif isinstance(value, str) and private:
            yield value


def registration_values(jit):
    values = {jit}
    decoded = decode(jit)
    config = read_json(decoded or jit)
    values.update(private_values(config))
    if isinstance(config, dict):
        for name, value in config.items():
            if isinstance(value, str) and PRIVATE.search(name):
                decoded_file = decode(value)
                values.add(decoded_file)
                values.update(private_values(read_json(decoded_file or value), private=True))
    secrets = {piece for value in values for piece in value.splitlines() if piece.strip()}
    return secrets | {json.dumps(value)[1:-1] for value in secrets}


def redact(line, secrets):
    try:
        text = line.removesuffix(b"\n").removesuffix(b"\r").decode()
    except UnicodeDecodeError:
        return REDACTED
    if text.endswith(": ACTIONS_RUNNER_INPUT_JITCONFIG: JIT registration required"):
        return b"[ciproxy] runner entry error: JIT registration required\n"
    if not text.strip():
        return b""
    if (any(character != "\t" and not character.isprintable() for character in text)
            or PRIVATE.search(text) or PRIVATE_URL.search(text) or OPAQUE.search(text)
            or any(value in text for value in secrets)):
        return REDACTED
    return text.encode() + b"\n"


def messages(source, secrets):
    retained = 0
    while line := source.readline(LINE_LIMIT + 1):
        if len(line) > LINE_LIMIT:
            break
        clean = redact(line, secrets)
        if retained + len(clean) > LOG_LIMIT:
            break
        if clean:
            retained += len(clean)
            yield clean
    else:
        return
    yield NOTICE
    while source.read(8192):
        pass


def forward(pending):
    first = pending.get()
    if first is None:
        return
    try:
        with subprocess.Popen(
            ["/usr/bin/systemd-cat", "--identifier=ciproxy-boat-runner-stderr",
             "--priority=warning", "--level-prefix=false"],
            stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env={},
        ) as journal:
            message = first
            while message is not None:
                journal.stdin.write(message)
                journal.stdin.flush()
                message = pending.get()
    except OSError:
        pass


def main():
    secrets = registration_values(os.environ.pop(JIT_VARIABLE, ""))
    pending = queue.SimpleQueue()
    threading.Thread(target=forward, args=(pending,)).start()
    try:
        for message in messages(sys.stdin.buffer, secrets):
            pending.put(message)
    finally:
        pending.put(None)


if __name__ == "__main__":
    main()
