# SPDX-License-Identifier: Apache-2.0
"""Select immutable tool releases while keeping their runtime defaults."""

import re


def azcopy_script(script, version):
    for architecture, suffix in (("arm64", "-arm64"), ("amd64", "")):
        script = script.replace("https://aka.ms/downloadazcopy-v10-linux" + suffix,
            "https://github.com/Azure/azure-storage-azcopy/releases/download/v" + version +
            "/azcopy_linux_" + architecture + "_" + version + ".tar.gz")
    return script


def bazel_script(script, version, bazelisk):
    script = script.replace("npm install -g @bazel/bazelisk",
                            "npm install -g @bazel/bazelisk@" + bazelisk + "\nexport USE_BAZEL_VERSION=" + version)
    return re.sub(r"(?m)^BAZEL_VERSION=.*$", 'BAZEL_VERSION="' + version + '"', script)


def ansible_injection(script, version):
    return script.replace("pipx inject $package ansible", "pipx inject $package ansible==" + version)


def docker_cache(script, images):
    for name, digest in images.items():
        original, pinned = "ghcr.io/" + name + ":latest", "ghcr.io/" + name + "@" + digest
        script = script.replace("docker pull " + original,
                                "docker pull " + pinned + "\n    docker tag " + pinned + " " + original)
    return script


def pypy_script(script, versions):
    cases = []
    for value in versions:
        python, pypy = value.removesuffix("]").split(" [PyPy ")
        major = ".".join(python.split(".")[:2])
        url = "https://downloads.python.org/pypy/pypy" + major + "-v" + pypy + "-linux64.tar.gz"
        cases.append('        ' + python + ') latest_major_pypy_version="' + url + '" ;;')
    replacement = '    case "$toolset_version" in\n' + "\n".join(cases) + "\n    esac"
    script = re.sub(r"(?ms)^    latest_major_pypy_version=\$\(.*?head -1\)", replacement, script)
    return script.replace("pypy_versions_json=$(curl -fsSL https://downloads.python.org/pypy/versions.json)\n", "")


def ruby_script(script):
    return re.sub(r"(?m)^    download_url=.*$",
                  '    download_url="https://github.com/ruby/ruby-builder/releases/download/ruby-${toolset_version}/ruby-${toolset_version}-ubuntu-${platform_version}-${arch}.tar.gz"', script)
