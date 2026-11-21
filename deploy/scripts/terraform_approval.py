"""The exact approval format used by mokly-cloud Terraform promotion."""

import json
import subprocess


class PolicyError(ValueError):
    """The saved plan or its approval does not satisfy promotion policy."""


def load_json(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise PolicyError(f"JSON repeats field {key}.")
            result[key] = value
        return result

    try:
        return json.loads(path.read_text(), object_pairs_hook=unique)
    except (OSError, json.JSONDecodeError) as error:
        raise PolicyError(f"Cannot read JSON: {error}") from error


def require_committed(repository, path):
    relative = path.relative_to(repository).as_posix()
    result = subprocess.run(
        ["git", "show", f"HEAD:{relative}"], cwd=repository, capture_output=True,
    )
    if result.returncode:
        raise PolicyError("Destructive approval must be committed in the merged revision.")
    if result.stdout != path.read_bytes():
        raise PolicyError("Destructive approval must be unmodified from the merged revision.")


def reason(value):
    if not isinstance(value, str) or not 10 <= len(value.strip()) <= 500:
        raise PolicyError("Destructive approval reason must contain 10 to 500 characters.")
    return value.strip()


def format_approval(document):
    """Keep the reference document's expanded objects and compact action arrays."""
    lines = json.dumps(document, indent=2).splitlines()
    result = []
    index = 0
    while index < len(lines):
        line = lines[index]
        if line.strip() == '"actions": [' and index + 1 < len(lines):
            end = index + 1
            while end < len(lines) and lines[end].strip() not in ("]", "],"):
                end += 1
            if end < len(lines) and all(
                item.strip().startswith('"') for item in lines[index + 1:end]
            ):
                values = [json.loads(item.strip().rstrip(",")) for item in lines[index + 1:end]]
                suffix = "," if lines[end].endswith(",") else ""
                result.append(line[:-1] + json.dumps(values) + suffix)
                index = end + 1
                continue
        result.append(line)
        index += 1
    return "\n".join(result) + "\n"
