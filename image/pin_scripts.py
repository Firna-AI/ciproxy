# SPDX-License-Identifier: Apache-2.0
"""Replace moving upstream release selections with shared concrete versions."""

import json
import re

from pin_release_tools import azcopy_script, bazel_script, docker_cache, ruby_script


def apply_scripts(upstream, pins):
    scripts = upstream / "images/ubuntu/scripts/build"
    versions, extra = pins["versions"], pins["extra"]

    def edit(name, old, new):
        path = scripts / name
        content = path.read_text()
        if old not in content:
            raise RuntimeError(f"Pinned software override does not match {name}: {old[:80]}")
        path.write_text(content.replace(old, new))

    def assignment(name, variable, value):
        path = scripts / name
        content = re.sub(r"(?m)^" + re.escape(variable) + r"=.*$", variable + "=" + value,
                         path.read_text())
        path.write_text(content)

    for filename, variable, version in [
        ("install-swift.sh", "swift_version", "Swift"),
        ("install-pulumi.sh", "version", "Pulumi"),
        ("install-julia.sh", "julia_version", "Julia"),
        ("install-java-tools.sh", "gradleLatestVersion", "Gradle"),
        ("install-kubernetes-tools.sh", "kubectl_minor_version", "Kubectl"),
        ("install-kubernetes-tools.sh", "minikube_version", "Minikube"),
    ]:
        value = versions[version]
        if variable == "kubectl_minor_version":
            value = "v" + ".".join(value.split(".")[:2])
        elif variable == "minikube_version":
            value = "v" + value
        assignment(filename, variable, json.dumps(value))
    assignment("install-java-tools.sh", "gradleDownloadUrl",
               json.dumps("https://services.gradle.org/distributions/gradle-" + versions["Gradle"] + "-bin.zip"))
    assignment("install-kubernetes-tools.sh", "exact_latest_url",
               json.dumps("https://github.com/kubernetes-sigs/kustomize/releases/tag/kustomize/v" + versions["Kustomize"]))
    edit("install-kubernetes-tools.sh", '| bash\n', '| bash -s -- --version v' + versions["Helm"] + "\n")
    edit("install-miniconda.sh", "Miniconda3-latest-Linux-x86_64.sh",
         "Miniconda3-py313_" + versions["Miniconda"] + "-1-Linux-x86_64.sh")
    edit("install-rust.sh", "--default-toolchain=stable", "--default-toolchain=" + versions["Rust"])
    edit("install-aws-tools.sh", "awscli-exe-linux-${cli_suffix}.zip",
         "awscli-exe-linux-${cli_suffix}-" + versions["AWS CLI"] + ".zip")
    edit("install-aws-tools.sh", "/plugin/latest/", "/plugin/" + versions["AWS CLI Session Manager Plugin"] + "/")
    edit("install-azure-devops-cli.sh", "az extension add -n azure-devops",
         "az extension add -n azure-devops --version " + versions["Azure CLI (azure-devops)"])
    edit("install-nodejs.sh", "/tj/n/master/", "/tj/n/v" + versions["n"] + "/")
    edit("install-homebrew.sh", "/Homebrew/install/master/", "/Homebrew/install/" + extra["brew_install_commit"] + "/")
    edit("install-homebrew.sh", '# Invoke shellenv',
         'git -C /home/linuxbrew/.linuxbrew/Homebrew fetch --depth 1 origin refs/tags/' + versions["Homebrew"] +
         '\ngit -C /home/linuxbrew/.linuxbrew/Homebrew checkout FETCH_HEAD\n# Invoke shellenv')
    assignment("install-awf.sh", "versions", "'" + " ".join(extra["awf"]) + "'")
    edit("install-dotnetcore-sdk.sh", "--tool-path '/etc/skel/.dotnet/tools'",
         "--tool-path '/etc/skel/.dotnet/tools' --version " + versions["nbgv"].split("+")[0])
    edit("Install-PowerShellModules.ps1", "Install-Module -Name PowerShellGet -Force",
         "Install-Module -Name PowerShellGet -RequiredVersion " + extra["PowerShellGet"] + " -Force")
    edit("Install-PowerShellModules.ps1", "Update-Module -Name PowerShellGet -Force", "")
    edit("install-php.sh", "php composer-setup.php", "php composer-setup.php --version=" + versions["Composer"])
    edit("install-php.sh", "phpunit-8.phar", "phpunit-" + versions["PHPUnit"] + ".phar")
    path = scripts / "install-bazel.sh"
    path.write_text(bazel_script(path.read_text(), versions["Bazel"], versions["Bazelisk"]))
    path = scripts / "install-azcopy.sh"
    path.write_text(azcopy_script(path.read_text(), versions["AzCopy"].split()[0]))
    path = scripts / "install-docker.sh"
    path.write_text(docker_cache(path.read_text(), pins["docker_images"]))
    path = scripts / "install-ruby.sh"
    path.write_text(ruby_script(path.read_text()))
    assignment("install-docker.sh", "aws_latest_release_url",
               json.dumps("https://api.github.com/repos/awslabs/amazon-ecr-credential-helper/releases/tags/v" + versions["Docker Amazon ECR Credential Helper"]))
    for name in ("install-nvm.sh", "install-zstd.sh"):
        path = scripts / name
        content = path.read_text()
        for repo, tag in pins["github"].items():
            content = content.replace("repos/" + repo + "/releases/latest", "repos/" + repo + "/releases/tags/" + tag)
        path.write_text(content)
    for name, variable in (("install-awf.sh", "releases"), ("install-java-tools.sh", "gradleJson"),
                           ("install-julia.sh", "json")):
        path = scripts / name
        path.write_text(re.sub(r"(?m)^" + variable + r"=\$\(curl[^\n]*\)\n", "", path.read_text()))
    helper = upstream / "images/ubuntu/scripts/helpers"
    (helper / "ciproxy-pins.json").write_text(json.dumps(pins, indent=2) + "\n")
    install = helper / "install.sh"
    install.write_text(install.read_text() + '''
# Shared exact release selections. Metadata can be prepared on the build host.
get_github_releases_by_version() {
    local repo=$1 version=${2:-} tag
    tag=$(jq -r --arg repo "$repo" '.github[$repo] // empty' "$HELPER_SCRIPTS/ciproxy-pins.json")
    if [[ -z "$tag" ]]; then tag="$version"; fi
    get_github_api_json "repos/${repo}/releases/tags/${tag}"
}
''')
