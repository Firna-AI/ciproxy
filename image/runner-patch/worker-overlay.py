#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Make the Worker overlay that install-worker.sh puts on the official runner.

The overlay holds the patched Worker, the package files that the patched
build raises over the official runner, and the runner's own deps file with
only those package entries changed. The official runner is self-contained,
so its deps file keeps its runtime pack. The patched build's deps file does
not, so the overlay never copies it.
"""

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import struct
import tarfile

DEPS = "Runner.Worker.deps.json"
MARKER = "stock-deps.sha256"
PIN = re.compile(r'<PackageReference Include="([^"]+)" Version="([^"]+)" />')


def version(text):
    if not re.fullmatch(r"\d+(\.\d+)*", text):
        raise RuntimeError(f"Unsupported package version: {text}")
    return tuple(int(part) for part in text.split("."))


def file_version(path):
    data = path.read_bytes()
    start = data.find(b"\xbd\x04\xef\xfe")
    if start < 0:
        raise RuntimeError(f"No file version in {path.name}")
    high, low = struct.unpack_from("<II", data, start + 8)
    return f"{high >> 16}.{high & 0xFFFF}.{low >> 16}.{low & 0xFFFF}"


def stock_deps(archive):
    with tarfile.open(archive, "r:gz") as runner:
        for member in runner.getmembers():
            if member.isfile() and member.name.removeprefix("./") == "bin/" + DEPS:
                return runner.extractfile(member).read()
    raise RuntimeError(f"No bin/{DEPS} in {archive}")


def packages(deps):
    return {key.split("/")[0]: key.split("/")[1]
            for key, value in deps["libraries"].items() if value["type"] == "package"}


def target(deps):
    return deps["targets"][deps["runtimeTarget"]["name"]]


def renamed(entries, old, new, value):
    return {(new if key == old else key): (value if key == old else entry)
            for key, entry in entries.items()}


def raised_packages(stock, patched):
    if stock["runtimeTarget"]["name"] != patched["runtimeTarget"]["name"]:
        raise RuntimeError("The patched Worker and the runner have different runtimes")
    old, new = packages(stock), packages(patched)
    raised = {}
    for name, value in sorted(new.items()):
        if name not in old:
            raise RuntimeError(f"The patched Worker adds package {name}, which the runner lacks")
        if value == old[name]:
            continue
        if version(value) < version(old[name]):
            raise RuntimeError(f"The patched Worker lowers {name} below {old[name]}")
        raised[name] = (old[name], value)
    return raised


def merge(stock, patched, raised):
    merged = json.loads(json.dumps(stock))
    runtime = merged["runtimeTarget"]["name"]
    for name, (old, new) in raised.items():
        entry = target(patched)[f"{name}/{new}"]
        stock_entry = target(stock)[f"{name}/{old}"]
        if set(entry) - {"dependencies", "runtime"} or set(entry["runtime"]) != set(stock_entry["runtime"]):
            raise RuntimeError(f"Package {name} changed its files; review the overlay")
        old_key, new_key = f"{name}/{old}", f"{name}/{new}"
        merged["targets"][runtime] = renamed(merged["targets"][runtime], old_key, new_key, entry)
        merged["libraries"] = renamed(merged["libraries"], old_key, new_key,
                                      patched["libraries"][new_key])
        for value in merged["targets"][runtime].values():
            if value.get("dependencies", {}).get(name) == old:
                value["dependencies"][name] = new
    resolved = packages(merged)
    for key, value in target(merged).items():
        for name, wanted in value.get("dependencies", {}).items():
            if name in resolved and resolved[name] != wanted:
                raise RuntimeError(f"{key} needs {name} {wanted}, but the overlay has {resolved[name]}")
    return merged


def check(output, merged, raised):
    pins = PIN.findall(Path(__file__).with_name("prepare-source.py").read_text(encoding="utf-8"))
    if not pins:
        raise RuntimeError("prepare-source.py pins no package")
    for name, pinned in pins:
        if packages(merged).get(name) != pinned or name not in raised:
            raise RuntimeError(f"The overlay does not raise {name} to {pinned}")
    for name, (_, new) in raised.items():
        for asset, facts in target(merged)[f"{name}/{new}"]["runtime"].items():
            found = file_version(output / Path(asset).name)
            if found != facts["fileVersion"]:
                raise RuntimeError(f"{Path(asset).name} has file version {found}, not {facts['fileVersion']}")


def write(output, worker_output, raw, merged, raised):
    files = [Path(asset).name for name, (_, new) in raised.items()
             for asset in target(merged)[f"{name}/{new}"]["runtime"]]
    output.mkdir(parents=True)
    for name in ["Runner.Worker.dll", *files]:
        shutil.copyfile(worker_output / name, output / name)
    (output / DEPS).write_text(json.dumps(merged, indent=2) + "\n", encoding="utf-8")
    check(output, merged, raised)
    (output / "install-files.txt").write_text("".join(f"{name}\n" for name in [*files, DEPS]))
    (output / "packages.txt").write_text(
        "".join(f"{name} {new}\n" for name, (_, new) in raised.items()))
    (output / MARKER).write_text(hashlib.sha256(raw).hexdigest() + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--worker-output", type=Path, required=True)
    parser.add_argument("--runner-archive", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() and not (args.output / MARKER).is_file():
        raise RuntimeError(f"{args.output} exists and is not a Worker overlay")
    raw = stock_deps(args.runner_archive)
    stock = json.loads(raw.decode("utf-8-sig"))
    patched = json.loads((args.worker_output / DEPS).read_text(encoding="utf-8-sig"))
    raised = raised_packages(stock, patched)
    merged = merge(stock, patched, raised)
    # Build in a side folder, so a failed run leaves no partial overlay.
    partial = args.output.with_name(args.output.name + ".partial")
    shutil.rmtree(partial, ignore_errors=True)
    try:
        write(partial, args.worker_output, raw, merged, raised)
    except BaseException:
        shutil.rmtree(partial, ignore_errors=True)
        raise
    if args.output.exists():
        shutil.rmtree(args.output)
    partial.rename(args.output)
    for name, (old, new) in raised.items():
        print(f"Worker overlay raises {name} from {old} to {new}")


if __name__ == "__main__":
    main()
