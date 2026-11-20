# SPDX-License-Identifier: Apache-2.0
"""Declare source inputs without depending on installed build output."""

import pathlib
import re


SOURCE = "/opt/ciproxy-build/runner-images"
UBUNTU = SOURCE + "/images/ubuntu"


def source_inputs(upstream, step):
    inputs = []
    environment = step["environment"]
    if any(name in environment for name in ("HELPER_SCRIPTS", "HELPER_SCRIPT_FOLDER")):
        inputs.append(UBUNTU + "/scripts/helpers")
    if "INSTALLER_SCRIPT_FOLDER" in environment:
        inputs.append(UBUNTU + "/toolsets/toolset-2404.json")
    if "script" in step:
        path = pathlib.Path(step["script"].replace(SOURCE, str(upstream)))
        content = path.read_text()
        if "/imagegeneration/post-generation" in content:
            inputs.append(UBUNTU + "/assets/post-gen")
        tests = re.findall(r'invoke_tests\s+"([^"\n]+)"', content)
        tests += re.findall(r'Invoke-PesterTests[^\n]*-TestFile\s+"([^"\n]+)"', content)
        if tests or "../tests/Helpers.psm1" in content:
            inputs.append(UBUNTU + "/scripts/tests/Helpers.psm1")
        for name in tests:
            inputs.append(UBUNTU + "/scripts/tests/" + name + ".Tests.ps1")
        for name in re.findall(r'\$(?:\{INSTALLER_SCRIPT_FOLDER\}|INSTALLER_SCRIPT_FOLDER)/([\w.-]+\.(?:sh|ps1))', content):
            inputs.append(UBUNTU + "/scripts/build/" + name)
        inputs.extend(re.findall(r"/opt/ciproxy-build/definition/[\w.-]+\.(?:py|sh)", content))
    else:
        commands = "\n".join(step.get("inline", []))
        if "Generate-SoftwareReport.ps1" in commands:
            inputs.extend([UBUNTU + "/scripts/docs-gen", SOURCE + "/helpers/software-report-base",
                           UBUNTU + "/scripts/helpers", UBUNTU + "/scripts/build"])
        if "RunAll-Tests.ps1" in commands:
            inputs.extend([UBUNTU + "/scripts/tests", UBUNTU + "/scripts/helpers"])
    return sorted(set(inputs))
