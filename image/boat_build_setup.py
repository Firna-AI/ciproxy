# SPDX-License-Identifier: Apache-2.0
"""Prepare owned image directories without losing existing metadata."""

import pathlib
import shlex


def merge_directory(source, destination):
    source_name, destination_name = shlex.quote(source), shlex.quote(destination)
    return "\n".join([
        "mkdir -p " + destination_name,
        "if [ -d " + source_name + " ]; then",
        "    cp -a " + shlex.quote(source + "/.") + " " + shlex.quote(destination + "/"),
        "    rm -rf " + source_name,
        "fi",
    ])


def copy_inputs(folder, steps):
    folder = pathlib.PurePosixPath(folder)
    inputs = []
    for step in steps:
        if step["type"] != "file" or step.get("skip"):
            continue
        destination = pathlib.PurePosixPath(step["destination"])
        for source in step["sources"]:
            target = destination if step.get("directory_contents") else destination / pathlib.PurePosixPath(source).name
            if folder == target or folder in target.parents or target in folder.parents:
                inputs.extend(step["sources"])
                break
    return inputs


def native_setup(commands, image_folder, previous_steps):
    result, inputs = [], []
    moves = [
        ("docs-gen", "SoftwareReport"),
        ("post-gen", "post-generation"),
    ]
    for command in commands:
        if command == "mkdir " + image_folder:
            result.append("mkdir -p " + shlex.quote(image_folder))
            continue
        for source, destination in moves:
            source, destination = image_folder + "/" + source, image_folder + "/" + destination
            if command == "mv " + source + " " + destination:
                result.append(merge_directory(source, destination))
                inputs.extend(copy_inputs(source, previous_steps))
                break
        else:
            result.append(command)
    return result, inputs
