# SPDX-License-Identifier: Apache-2.0
"""Relocate literal installer scratch paths without changing runtime data."""

import re


TEMPORARY = "/opt/ciproxy-build/tmp"
WORDS = re.compile(r'''(?:[^\s'"\\|&;()<>]+|\\.|"(?:\\.|[^"\\])*"|'[^']*')+''')
PATH = re.compile(r'''/tmp(?=$|[/"'])''')
PREFIX = re.compile(r'''^(?:(?:\$?[A-Za-z_][\w:]*|--?[\w-]+)=)?["']?$''')
PERSISTED = {"set_etc_environment_variable", "replace_etc_environment_variable",
             "prepend_etc_environment_path", "append_etc_environment_path",
             "add-globalenvironmentvariable", "set-content", "add-content", "out-file"}


def heredocs(line):
    result, quote, escaped, index = [], None, False, 0
    pattern = re.compile(r'''<<(-?)\s*(?:'([^']+)'|"([^"]+)"|([A-Za-z_]\w*))''')
    while index < len(line):
        char = line[index]
        if escaped:
            escaped = False
        elif char == "\\" and quote != "'":
            escaped = True
        elif quote:
            if char == quote:
                quote = None
        elif char in "\"'":
            quote = char
        elif char == "#" and (index == 0 or line[index - 1].isspace()):
            break
        elif line.startswith("<<", index):
            match = pattern.match(line, index)
            if match:
                result.append((next(value for value in match.groups()[1:] if value), bool(match[1])))
                index = match.end()
                continue
        index += 1
    return result


def path_word(word, temporary):
    if re.search(r"[A-Za-z][A-Za-z0-9+.-]*://", word):
        return word
    match = PATH.search(word)
    if match and PREFIX.fullmatch(word[:match.start()]):
        return word[:match.start()] + temporary + word[match.end():]
    return word


def command_line(line, temporary):
    result, previous, command = [], 0, None
    for match in WORDS.finditer(line):
        gap, word = line[previous:match.start()], match[0]
        result.append(gap)
        if word.startswith("#"):
            result.append(line[match.start():])
            return "".join(result)
        if re.search(r"[|;&]", gap):
            command = None
        if command is None and word not in {"sudo", "env", "then", "do", "else"}:
            if not re.match(r"\$?[A-Za-z_][\w:]*=", word):
                command = word.lower()
        redirected = bool(re.search(r">\s*$", gap))
        data = command in PERSISTED or (command in {"echo", "printf"} and not redirected) or "<<<" in gap
        result.append(word if data else path_word(word, temporary))
        previous = match.end()
    result.append(line[previous:])
    return "".join(result)


def relocate_temp_paths(script, temporary=TEMPORARY):
    result, pending = [], []
    for line in script.splitlines(keepends=True):
        if pending:
            result.append(line)
            delimiter, strip_tabs = pending[0]
            ending = line.rstrip("\r\n")
            if (ending.lstrip("\t") if strip_tabs else ending) == delimiter:
                pending.pop(0)
            continue
        result.append(command_line(line, temporary))
        pending.extend(heredocs(line))
    return "".join(result)
