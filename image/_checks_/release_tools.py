# SPDX-License-Identifier: Apache-2.0
"""Execute release selection and Docker cache preparation with command fixtures."""

import json
import os
import pathlib
import subprocess
import sys
import tempfile

from pin_release_tools import azcopy_script, bazel_script, docker_cache, pypy_script, ruby_script


def tools_fixture(workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    tools = root / "tools"
    tools.mkdir()
    events = root / "events.jsonl"
    environment = {**os.environ, "PATH": str(tools) + os.pathsep + os.environ["PATH"],
                   "TOOL_EVENTS": str(events)}
    return root, tools, events, environment


def command(tools, name, body):
    path = tools / name
    path.write_text(f"#!{sys.executable}\nimport json,os,sys\n" + body)
    path.chmod(0o755)


def record():
    return "with open(os.environ['TOOL_EVENTS'],'a') as f: f.write(json.dumps(sys.argv[1:])+'\\n')\n"


def check_azcopy(case, workspace):
    root, tools, events, environment = tools_fixture(workspace)
    command(tools, "curl", record())
    script = azcopy_script('azcopy_download_url="https://aka.ms/downloadazcopy-v10-linux"\n'
                           'curl -fsSL "$azcopy_download_url"\n', "10.34.1")
    result = subprocess.run(["bash", "-e", "-c", script], env=environment, capture_output=True, text=True)
    case.assertEqual(result.returncode, 0, result.stderr)
    case.assertEqual(json.loads(events.read_text()), ["-fsSL",
        "https://github.com/Azure/azure-storage-azcopy/releases/download/v10.34.1/azcopy_linux_amd64_10.34.1.tar.gz"])


def check_bazel(case, workspace):
    root, tools, events, environment = tools_fixture(workspace)
    command(tools, "npm", record())
    command(tools, "bazel", record() +
            "with open(os.environ['BAZEL_REQUESTS'],'a') as f: f.write(str(os.environ.get('USE_BAZEL_VERSION'))+'\\n')\n"
            "print('bazel 99.0.0' if '--version' in sys.argv else 'Build label: 99.0.0')\n")
    environment["BAZEL_REQUESTS"] = str(root / "requests")
    environment["RUNTIME_ENV"] = str(root / "runtime.env")
    environment.pop("USE_BAZEL_VERSION", None)
    script = '''set_etc_environment_variable() { printf '%s=%s\\n' "$1" "$2" >> "$RUNTIME_ENV"; }
npm install -g @bazel/bazelisk
bazel version
BAZEL_VERSION=$(bazel --version | grep "Build label:" | awk '{print $3}')
set_etc_environment_variable USE_BAZEL_FALLBACK_VERSION "silent:${BAZEL_VERSION}"
'''
    result = subprocess.run(["bash", "-e", "-c", bazel_script(script, "9.3.1", "1.28.1")],
                            env=environment, capture_output=True, text=True)
    case.assertEqual(result.returncode, 0, result.stderr)
    case.assertEqual((root / "runtime.env").read_text(), "USE_BAZEL_FALLBACK_VERSION=silent:9.3.1\n")
    case.assertEqual(set((root / "requests").read_text().splitlines()), {"9.3.1"})


def check_docker(case, workspace):
    root, tools, events, environment = tools_fixture(workspace)
    command(tools, "docker", record())
    definition = pathlib.Path(__file__).resolve().parents[1]
    images = json.loads((definition / "software-pins.json").read_text())["docker_images"]
    script = "\n".join("docker pull ghcr.io/" + name + ":latest" for name in images)
    result = subprocess.run(["bash", "-e", "-c", docker_cache(script, images)],
                            env=environment, capture_output=True, text=True)
    case.assertEqual(result.returncode, 0, result.stderr)
    observed = [json.loads(line) for line in events.read_text().splitlines()]
    case.assertEqual([args for args in observed if args[0] == "pull"],
                     [["pull", "ghcr.io/" + name + "@" + digest] for name, digest in images.items()])
    case.assertEqual([args for args in observed if args[0] == "tag"],
                     [["tag", "ghcr.io/" + name + "@" + digest, "ghcr.io/" + name + ":latest"]
                      for name, digest in images.items()])


def check_pypy(case, workspace):
    root, tools, events, environment = tools_fixture(workspace)
    command(tools, "curl", "print('[]')\n")
    command(tools, "jq", "print('https://example.invalid/newer-pypy-with-same-python.tar.gz')\n")
    script = '''install_pypy() { printf '%s\\n' "$1" >> "$TOOL_EVENTS"; }
get_toolset_value() { printf '3.9.19\\n'; }
pypy_versions_json=$(curl -fsSL https://downloads.python.org/pypy/versions.json)
toolset_versions=$(get_toolset_value ignored)
for toolset_version in $toolset_versions; do
    latest_major_pypy_version=$(echo $pypy_versions_json |
        jq -r ignored | head -1)
    install_pypy $latest_major_pypy_version
done
'''
    result = subprocess.run(["bash", "-e", "-c", pypy_script(script, ["3.9.19 [PyPy 7.3.16]"])],
                            env=environment, capture_output=True, text=True)
    case.assertEqual(result.returncode, 0, result.stderr)
    case.assertEqual(events.read_text(), "https://downloads.python.org/pypy/pypy3.9-v7.3.16-linux64.tar.gz\n")


def check_ruby(case, workspace):
    root, tools, events, environment = tools_fixture(workspace)
    script = '''resolve_github_release_asset_url() { printf 'https://example.invalid/releases/tags/3.2.11\\n'; }
platform_version=24.04
arch=x64
for toolset_version in 3.2.11; do
    download_url=$(resolve_github_release_asset_url ruby/ruby-builder ignored "$toolset_version")
    printf '%s\\n' "$download_url" >> "$TOOL_EVENTS"
done
'''
    result = subprocess.run(["bash", "-e", "-c", ruby_script(script)], env=environment,
                            capture_output=True, text=True)
    case.assertEqual(result.returncode, 0, result.stderr)
    case.assertEqual(events.read_text(), "https://github.com/ruby/ruby-builder/releases/download/ruby-3.2.11/ruby-3.2.11-ubuntu-24.04-x64.tar.gz\n")
