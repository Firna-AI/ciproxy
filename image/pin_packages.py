# SPDX-License-Identifier: Apache-2.0
"""Pin package-manager selections shared by machine and native builds."""

import json
import re

from pin_release_tools import ansible_injection


def apt_preferences(packages):
    return "\n".join(f"Package: {name}\nPin: version {version}\nPin-Priority: 1001\n"
                     for name, version in packages.items())


def apt_install_commands(script):
    return script.replace("apt-get install --no-install-recommends $package", '''install_package=$package
    # Keep the logical tool name for the upstream CLI checks.
    case "$package" in
        netcat) install_package=netcat-openbsd ;;
    esac
    apt-get install --no-install-recommends "$install_package"''')


def apply_packages(upstream, pins, apt):
    scripts = upstream / "images/ubuntu/scripts/build"
    versions = pins["versions"]

    def edit(name, old, new):
        path = scripts / name
        content = path.read_text()
        if old not in content:
            raise RuntimeError(f"Pinned package override does not match {name}: {old[:80]}")
        path.write_text(content.replace(old, new))

    preferences = apt_preferences(apt)
    edit("install-java-tools.sh", r"temurin-${java_version}-jdk=\*", "temurin-${java_version}-jdk")
    edit("configure-apt.sh", "source $HELPER_SCRIPTS/os.sh",
         "source $HELPER_SCRIPTS/os.sh\ncat > /etc/apt/preferences.d/ciproxy-software <<'CIPROXY_PINS'\n" +
         preferences + "\nCIPROXY_PINS\n")
    apt_common = scripts / "install-apt-common.sh"
    apt_common.write_text(apt_install_commands(apt_common.read_text()))
    edit("install-postgresql.sh", "https://apt.postgresql.org/pub/repos/apt/",
         "https://apt-archive.postgresql.org/pub/repos/apt/")
    edit("install-postgresql.sh", "-pgdg main", "-pgdg-archive main")
    edit("install-nodejs.sh", "npm install -g $node_modules", "npm install -g " +
         " ".join(name + "@" + version for name, version in pins["npm"].items() if name != "@bazel/bazelisk"))
    edit("install-ruby.sh", "gem install --no-document $gem",
         'version=$(jq -r --arg name "$gem" '\
         "'.gem_versions[$name]' \"$HELPER_SCRIPTS/ciproxy-pins.json\")\n"
         '        gem install --no-document "$gem" --version "$version"')
    pins["gem_versions"] = {"multi_json": pins["extra"]["multi_json"], "fastlane": versions["Fastlane"]}
    pins["pipx_versions"] = {"yamllint": versions["yamllint"], "ansible-core": versions["Ansible"]}
    edit("install-pipx-packages.sh", "pipx install $package",
         'version=$(jq -r --arg name "$package" '\
         "'.pipx_versions[$name]' \"$HELPER_SCRIPTS/ciproxy-pins.json\")\n"
         '    pipx install "$package==$version"')
    pipx = scripts / "install-pipx-packages.sh"
    pipx.write_text(ansible_injection(pipx.read_text(), pins["extra"]["ansible"]))
    edit("install-pypy.sh", "--ignore-installed pip", "--ignore-installed pip==" + versions["Pip"])
    edit("install-packer.sh", "/releases/packer/latest", "/releases/packer/" + versions["Packer"])
    edit("install-google-chrome.sh", "https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb",
         "https://dl.google.com/linux/chrome/deb/pool/main/g/google-chrome-stable/google-chrome-stable_" + versions["Google Chrome"] + "-1_amd64.deb")
    for variable, value in {
        "chromedriver_version": versions["ChromeDriver"],
        "chromedriver_url": "https://storage.googleapis.com/chrome-for-testing-public/" + versions["ChromeDriver"] + "/linux64/chromedriver-linux64.zip",
        "chromium_revision": pins["extra"]["chromium_revision"],
    }.items():
        path = scripts / "install-google-chrome.sh"
        path.write_text(re.sub(r"(?m)^" + variable + r"=.*$", variable + "=" + json.dumps(value), path.read_text()))
    path = scripts / "install-microsoft-edge.sh"
    path.write_text(re.sub(r"(?m)^edgedriver_latest_version=.*$", "edgedriver_latest_version=" +
                          json.dumps(versions["Microsoft Edge WebDriver"]), path.read_text()))
    path.write_text(re.sub(r"(?m)^edgedriver_version_url=.*\n", "", path.read_text()))
    path = scripts / "install-google-chrome.sh"
    content = re.sub(r"(?ms)^get_chromium_revision\(\) \{.*?^\}\n", "", path.read_text())
    content = re.sub(r"(?m)^(?:CHROME_VERSIONS_URL|chrome_versions_json|chrome_revision)=.*\n", "", content)
    path.write_text(content)
