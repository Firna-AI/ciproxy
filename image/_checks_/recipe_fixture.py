# SPDX-License-Identifier: Apache-2.0
"""Render the shared recipe with offline upstream installer fixtures."""

import json
import pathlib
import shutil
import tempfile

from pin_recipe import render
from _checks_.boat_owners import PINNED_TEST


def rendered(workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    source = root / "upstream"
    ubuntu = source / "images/ubuntu"
    scripts = ubuntu / "scripts/build"
    helpers = ubuntu / "scripts/helpers"
    scripts.mkdir(parents=True)
    helpers.mkdir()
    fixtures = pathlib.Path(__file__).with_name("fixtures") / "runner-images"
    for name in ("install-pypy.sh", "install-java-tools.sh", "install-nodejs.sh"):
        shutil.copy2(fixtures / name, scripts / name)
    shutil.copy2(fixtures / "install.sh", helpers / "install.sh")
    bodies = {
        "configure-environment.sh": ("# Change waagent entries\n# Add localhost alias\n"
            "# Relax root filesystem durability\n# Create symlink for tests\nmkdir $AGENT_TOOLSDIRECTORY\n"
            "ln -s $HELPER_SCRIPTS/invoke-tests.sh /usr/local/bin/invoke_tests\n"),
        "configure-apt-sources.sh": "touch /etc/apt/apt-mirrors.txt\n",
        "configure-system.sh": "mv -f /imagegeneration/post-generation /opt\n",
        "cleanup.sh": "",
        "configure-apt.sh": "source $HELPER_SCRIPTS/os.sh\n",
        "install-apt-common.sh": "apt-get install --no-install-recommends $package\n",
        "install-postgresql.sh": "https://apt.postgresql.org/pub/repos/apt/ -pgdg main\n",
        "install-ruby.sh": "gem install --no-document $gem\n    download_url=old\n",
        "install-pipx-packages.sh": "pipx install $package\npipx inject $package ansible\n",
        "install-packer.sh": "https://example.invalid/releases/packer/latest\n",
        "install-google-chrome.sh": ("https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb\n"
            "chromedriver_version=old\nchromedriver_url=old\nchromium_revision=old\n"
            "get_chromium_revision() {\n    :\n}\nCHROME_VERSIONS_URL=old\n"),
        "install-microsoft-edge.sh": "edgedriver_latest_version=old\nedgedriver_version_url=old\n",
        "install-codeql-bundle.sh": "# Retrieve the latest major\nold\nbundle_tag_name=old\n",
        "install-copilot-cli.sh": "# Pin to the catch-all\nold\n# Download the release tarball\n",
        "install-dotnetcore-sdk.sh": "sdks=()\nold\n## Download installer\n--tool-path '/etc/skel/.dotnet/tools'\n",
        "install-haskell.sh": "# Install GHCup\nold\nchmod -R 777 /tmp\ncurl -fsSL https://get.haskellstack.org/ | bash\n",
        "install-rust.sh": "curl -fsSL https://sh.rustup.rs | sh -s -- --default-toolchain=stable\n",
        "install-julia.sh": "julia_version=old\njson=$(curl old)\njulia_tar_url=old\n",
        "install-android-sdk.sh": '    ndk_version=$(old)\n    echo "$ndk_version"\navailable_platforms=(old)\n# Install components\n',
        "install-swift.sh": "swift_version=old\n",
        "install-pulumi.sh": "version=old\n",
        "install-kubernetes-tools.sh": "kubectl_minor_version=old\nminikube_version=old\nexact_latest_url=old\ncurl old | bash\n",
        "install-miniconda.sh": "Miniconda3-latest-Linux-x86_64.sh\n",
        "install-aws-tools.sh": "awscli-exe-linux-${cli_suffix}.zip\n/plugin/latest/\n",
        "install-azure-devops-cli.sh": "az extension add -n azure-devops\n",
        "install-homebrew.sh": "/Homebrew/install/master/\n# Invoke shellenv\n",
        "install-awf.sh": "versions=old\nreleases=$(curl old)\n",
        "Install-PowerShellModules.ps1": "Install-Module -Name PowerShellGet -Force\nUpdate-Module -Name PowerShellGet -Force\n",
        "install-php.sh": "php composer-setup.php\nphpunit-8.phar\n",
        "install-bazel.sh": "npm install -g @bazel/bazelisk\nBAZEL_VERSION=old\n",
        "install-azcopy.sh": "https://aka.ms/downloadazcopy-v10-linux\n",
        "install-docker.sh": 'aws_latest_release_url=old\ngroupmod -g "$gid" docker\n# Docker daemon takes time\n',
        "install-nvm.sh": "",
        "install-zstd.sh": "",
    }
    for name, body in bodies.items():
        (scripts / name).write_text(body)
    (ubuntu / "assets/post-gen").mkdir(parents=True)
    (ubuntu / "assets/post-gen/cleanup-logs.sh").write_text("")
    (ubuntu / "scripts/tests").mkdir()
    (ubuntu / "scripts/tests/Tools.Tests.ps1").write_text(PINNED_TEST)
    (ubuntu / "scripts/tests/System.Tests.ps1").write_text("")
    toolset = {"toolcache": [{"name": name, "versions": ["1.*"]} for name in ("Python", "node", "Ruby", "PyPy")],
               "java": {"versions": ["17"], "default": "17", "maven": "3"},
               "cmake": {"version": "3.31.6"}, "pwsh": {"version": "7.6"}, "node": {"default": "22"}, "selenium": {"version": "4"},
               "node_modules": [], "powershellModules": [], "docker": {"components": [], "plugins": []}}
    toolset["toolcache"].append({"name": "go", "versions": ["1.24.*"], "default": "1.24.*"})
    (ubuntu / "toolsets").mkdir()
    (ubuntu / "toolsets/toolset-2404.json").write_text(json.dumps(toolset))
    output = root / "rendered"
    pins = render(source, output)
    return root, output, pins
