"""Select and hash the exact, tracked context sent to the image builder."""

import fnmatch
import hashlib
from functools import lru_cache
from pathlib import Path
import subprocess


DOCKERFILE = "crates/ciproxy-control-plane/Dockerfile"
PLATFORM = "linux/amd64"
BUILD_TOOLS = (
    ".github/workflows/deploy-control-plane-image.yml",
    "deploy/scripts/image_context.py",
    "deploy/scripts/control_plane_image.py",
    "deploy/scripts/image_registry.py",
    "deploy/scripts/image_outputs.py",
    "deploy/scripts/install-image-tools.sh",
    "deploy/scripts/start-buildkit.sh",
)


def tracked_files(source, revision):
    actual = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    if actual != revision:
        raise ValueError("Image source checkout must match the exact push commit.")
    listing = subprocess.check_output(["git", "-C", str(source), "ls-tree", "-rz", "HEAD"])
    entries = []
    for entry in listing.split(b"\0"):
        if entry:
            metadata, path = entry.split(b"\t", 1)
            mode, kind, object_id = metadata.split()
            if kind == b"blob":
                entries.append((path.decode(), mode.decode(), object_id))
    batch = subprocess.run(
        ["git", "-C", str(source), "cat-file", "--batch"],
        input=b"\n".join(item[2] for item in entries) + b"\n",
        check=True, stdout=subprocess.PIPE,
    ).stdout
    files = {}
    offset = 0
    for path, mode, _ in entries:
        end = batch.index(b"\n", offset)
        size = int(batch[offset:end].split()[-1])
        files[path] = (mode, batch[end + 1:end + 1 + size])
        offset = end + 2 + size
    return files


@lru_cache(maxsize=None)
def matches(path, pattern):
    if not pattern:
        return not path
    if pattern[0] == "**":
        return matches(path, pattern[1:]) or bool(path and matches(path[1:], pattern))
    return bool(path and fnmatch.fnmatchcase(path[0], pattern[0])
                and matches(path[1:], pattern[1:]))


def select(files):
    rules = []
    for line in files[".dockerignore"][1].decode().splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        include = line.strip().startswith("!")
        pattern = line.strip()[1:] if include else line.strip()
        # Directory reopen entries permit traversal. Only file rules add bytes.
        if include and pattern.endswith("/"):
            continue
        if "\\" in pattern or not pattern:
            raise ValueError("Image context needs an unescaped .dockerignore pattern.")
        parts = tuple(part for part in pattern.strip("/").split("/") if part != ".")
        rules.append((include, parts))
    selected = {}
    for path, item in files.items():
        parts = tuple(path.split("/"))
        included = True
        for include, pattern in rules:
            if any(matches(parts[:length], pattern) for length in range(1, len(parts) + 1)):
                included = include
        if included or path in (".dockerignore", DOCKERFILE):
            selected[path] = item
    return selected


def source_context(source, revision):
    selected = select(tracked_files(source, revision))
    digest = hashlib.sha256()
    tools = Path(__file__).resolve().parents[2]
    records = {"source/" + name: item for name, item in selected.items()}
    records.update({"tools/" + name: ("100644", (tools / name).read_bytes()) for name in BUILD_TOOLS})
    records["platform"] = ("100644", PLATFORM.encode())
    for name, (mode, data) in sorted(records.items()):
        for part in (name.encode(), mode.encode(), data):
            digest.update(len(part).to_bytes(8, "big"))
            digest.update(part)
    return digest.hexdigest(), selected


def materialize(files, destination):
    for name, (mode, data) in sorted(files.items()):
        path = destination / name
        path.parent.mkdir(parents=True, exist_ok=True)
        if mode == "120000":
            path.symlink_to(data.decode())
        else:
            path.write_bytes(data)
            path.chmod(int(mode, 8) & 0o777)
