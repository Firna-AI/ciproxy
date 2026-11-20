# SPDX-License-Identifier: Apache-2.0
"""Apply common exact software selections to the runner-images toolset."""

import json


def apply_toolset(upstream, pins):
    path = upstream / "images/ubuntu/toolsets/toolset-2404.json"
    toolset = json.loads(path.read_text())
    versions = pins["versions"]
    names = {"Python": "Python", "node": "Node.js", "go": "Go", "Ruby": "Ruby"}
    for tool in toolset["toolcache"]:
        if tool["name"] in names:
            tool["versions"] = versions[names[tool["name"]]]
            if "default" in tool:
                tool["default"] = tool["versions"][0]
        elif tool["name"] == "CodeQL":
            tool["versions"] = [versions["CodeQL Action Bundle"]]
        elif tool["name"] == "PyPy":
            tool["versions"] = [value.split(" ")[0] for value in versions["PyPy"]]
    toolset["java"]["maven"] = versions["Maven"]
    toolset["pwsh"]["version"] = versions["PowerShell"]
    toolset["node"]["default"] = versions["Node.js"][0]
    toolset["selenium"]["version"] = versions["Selenium server"]
    toolset["vcpkg"] = {"commit": pins["extra"]["vcpkg_commit"]}
    for module in toolset["powershellModules"]:
        module["versions"] = versions[module["name"]]
    for component in toolset["docker"]["components"]:
        if component["package"] == "containerd.io":
            component["version"] = pins["extra"]["containerd"]
    for plugin in toolset["docker"]["plugins"]:
        if plugin["plugin"] == "buildx":
            plugin["version"] = versions["Docker-Buildx"]
    path.write_text(json.dumps(toolset, indent=2) + "\n")
