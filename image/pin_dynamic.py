# SPDX-License-Identifier: Apache-2.0
"""Fix dynamic SDK, browser and runtime channel selections."""

import json
import re

from pin_release_tools import pypy_script


def apply_dynamic(upstream, pins):
    scripts = upstream / "images/ubuntu/scripts/build"
    versions, extra = pins["versions"], pins["extra"]

    def section(name, start, end, replacement):
        path = scripts / name
        text = path.read_text()
        first, last = text.index(start), text.index(end, text.index(start))
        path.write_text(text[:first] + replacement + "\n" + text[last:])

    section("install-codeql-bundle.sh", "# Retrieve the latest major", "bundle_tag_name=",
            "bundle_version=" + json.dumps(versions["CodeQL Action Bundle"]))
    section("install-copilot-cli.sh", "# Pin to the catch-all", "# Download the release tarball",
            "copilot_version=" + json.dumps(extra["copilot"]))
    section("install-dotnetcore-sdk.sh", "sdks=()", "## Download installer",
            "sorted_sdks='" + " ".join(versions[".NET Core SDK"]) + "'")
    section("install-haskell.sh", "# Install GHCup", 'chmod -R 777',
            'mkdir -p "$ghcup_bin"\n'
            'curl -fsSL https://downloads.haskell.org/~ghcup/' + versions["GHCup"] +
            '/x86_64-linux-ghcup-' + versions["GHCup"] + ' -o "$ghcup_bin/ghcup"\n'
            'chmod +x "$ghcup_bin/ghcup"\n'
            'export PATH="$ghcup_bin:$PATH"\nprepend_etc_environment_path "$ghcup_bin"\n'
            'ghcup install ghc ' + versions["GHC"] + '\nghcup set ghc ' + versions["GHC"] +
            '\nghcup install cabal ' + versions["Cabal"])
    path = scripts / "install-haskell.sh"
    path.write_text(path.read_text().replace("curl -fsSL https://get.haskellstack.org/ | bash",
        'curl -fsSL https://github.com/commercialhaskell/stack/releases/download/v' + versions["Stack"] +
        '/stack-' + versions["Stack"] + '-linux-x86_64.tar.gz | '
        'tar -xz --strip-components=1 -C /usr/local/bin --wildcards "*/stack"'))
    path = scripts / "install-rust.sh"
    path.write_text(path.read_text().replace("curl -fsSL https://sh.rustup.rs | sh -s --",
        "curl -fsSL https://static.rust-lang.org/rustup/archive/" + versions["Rustup"] +
        "/x86_64-unknown-linux-gnu/rustup-init -o /tmp/rustup-init\n"
        "chmod +x /tmp/rustup-init\n/tmp/rustup-init"))
    path = scripts / "install-julia.sh"
    path.write_text(re.sub(r"(?m)^julia_tar_url=.*$", "julia_tar_url=" + json.dumps(
        "https://julialang-s3.julialang.org/bin/linux/x64/" + ".".join(versions["Julia"].split(".")[:2]) +
        "/julia-" + versions["Julia"] + "-linux-x86_64.tar.gz"), path.read_text()))
    path = scripts / "install-android-sdk.sh"
    text = path.read_text()
    first, last = text.index('    ndk_version=$('), text.index('\n    echo "$ndk_version"')
    ndks = {"27": "27.3.13750724", "29": "29.0.14206865", "30": "30.0.16248370"}
    text = text[:first] + "    case $major_version in\n" + "\n".join(
        f"        {major}) ndk_version={version} ;;" for major, version in ndks.items()) + \
        "\n    esac\n" + text[last:]
    first, last = text.index('available_platforms=('), text.index('# Install components')
    suffix = text[last:]
    platforms = ['34', '34-ext8', '34-ext10', '34-ext11', '34-ext12', '35', '35-ext14', '35-ext15',
                 '36', '36-ext18', '36-ext19', '36.1', '37.0', '37.1', '37.2',
                 '37.2-beta1', '37.2-beta2', '37.2-beta3']
    text = text[:first] + "components+=(" + " ".join('"platforms;android-' + value + '"' for value in platforms) + ")\n" + \
        'components+=("build-tools;34.0.0" "build-tools;35.0.0" "build-tools;35.0.1" '
    text += '"build-tools;36.0.0" "build-tools;36.1.0" "build-tools;37.0.0")\n'
    text += 'platform_tools=$(download_with_retry "https://dl.google.com/android/repository/platform-tools_r37.0.1-linux.zip")\n'
    text += 'unzip -q "$platform_tools" -d "$ANDROID_SDK_ROOT"\n' + suffix
    path.write_text(text)
    path = scripts / "install-pypy.sh"
    path.write_text(pypy_script(path.read_text(), versions["PyPy"]))
