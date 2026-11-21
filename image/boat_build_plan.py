# SPDX-License-Identifier: Apache-2.0
"""Translate the pinned Packer provisioners into native build steps."""

import json
import pathlib
import re

from boat_build_inputs import source_inputs
from boat_build_setup import native_setup


VARIABLES = {
    "image_folder": "/imagegeneration",
    "helper_script_folder": "/imagegeneration/helpers",
    "installer_script_folder": "/imagegeneration/installers",
    "imagedata_file": "/imagegeneration/imagedata.json",
    "image_version": "20261101.360.1",
    "image_os": "ubuntu24",
}


def provisioners(recipe):
    """Read HCL blocks without treating braces inside strings as blocks."""
    pattern = re.compile(r'provisioner\s+"(shell|file)"\s*\{')
    offset = 0
    while match := pattern.search(recipe, offset):
        depth, quoted, escaped = 1, False, False
        end = match.end()
        while depth:
            char = recipe[end]
            if escaped:
                escaped = False
            elif quoted and char == "\\":
                escaped = True
            elif char == '"':
                quoted = not quoted
            elif not quoted:
                depth += (char == "{") - (char == "}")
            end += 1
        yield match.group(1), recipe[match.end():end - 1]
        offset = end


def attribute(block, name, default=None):
    match = re.search(r"\b" + name + r"\s*=\s*", block)
    if match is None:
        return default
    return json.JSONDecoder().raw_decode(block[match.end():])[0]


def render(value):
    for name, replacement in VARIABLES.items():
        value = value.replace("${var." + name + "}", replacement)
    return value.replace("${path.root}", "/opt/ciproxy-build/runner-images/images/ubuntu/templates")


def make_plan(upstream):
    recipe = upstream / "images/ubuntu/templates/build.ubuntu-24_04.pkr.hcl"
    steps = []
    for number, (kind, block) in enumerate(provisioners(recipe.read_text()), 1):
        base = {"provisioner": number, "type": kind,
                "environment": dict(value.split("=", 1) for value in
                                    map(render, attribute(block, "environment_vars", [])))}
        if kind == "file":
            step = {**base, "sources": list(map(render, attribute(block, "sources", [
                attribute(block, "source")]))), "destination": render(attribute(block, "destination")),
                "direction": attribute(block, "direction", "upload")}
            step["directory_contents"] = not step["destination"].endswith("/") and len(step["sources"]) == 1
            if step["direction"] == "download":
                step["skip"] = "The report stays in the native build artifacts and is read through the command API."
            steps.append(step)
            continue
        scripts = attribute(block, "scripts", [attribute(block, "script")])
        for script in scripts:
            if script is not None:
                step = {**base, "script": render(script),
                              "interpreter": "pwsh" if "pwsh -f" in block else "bash",
                              "user": "user" if '/bin/sh -c' in block else "root"}
                if script.endswith("configure-apt-mock.sh"):
                    step["skip"] = "The apt wrapper is an Azure extension workaround; Boat uses native apt exit codes."
                steps.append(step)
        inline = attribute(block, "inline")
        if inline:
            commands, dependencies = native_setup(list(map(render, inline)), VARIABLES["image_folder"], steps)
            step = {**base, "inline": commands, "user": "root", "inputs": dependencies}
            if any("waagent -force" in command for command in inline):
                step["skip"] = "Azure deprovisioning would remove Boat's provider account and controller access."
            elif any("sudo reboot" in command for command in inline):
                step["skip"] = "Boat restores files onto its own running system; its provider owns boot and kernel setup."
            steps.append(step)
    for index, step in enumerate(steps, 1):
        if step["type"] == "shell":
            step["inputs"] = sorted(set(step.get("inputs", []) + source_inputs(upstream, step)))
            step["working_directory"] = "/opt/ciproxy-build/tmp"
        name = pathlib.PurePosixPath(step.get("script", "")).name
        name = name or ("copy" if step["type"] == "file" else "inline")
        step["id"] = f"{index:03d}-{name}"
    return {"schema": 1, "steps": steps}
