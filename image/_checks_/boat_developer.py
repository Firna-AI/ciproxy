# SPDX-License-Identifier: Apache-2.0
"""Run developer cleanup against private files and package-tool boundaries."""

import hashlib
import json
import os
import pathlib
import shutil
import stat
import subprocess
import sys
import tempfile
import types
from unittest.mock import patch

from boat_build_report import collect


DEFINITION = pathlib.Path(__file__).resolve().parents[1]
RECORD = pathlib.Path("opt/ciproxy-build/provider-developer-environment.json")
PROVIDER = (
    "agent", "claude", "codex", "cursor-agent", "hermes", "hermes-acp", "kimi", "openclaw",
    "opencode", "pi", "prime-agent", "vibe", "vibe-acp", "cua-driver", "ariana-fswatch",
    "boat", "box", "ascii-desktop", "ascii-agent", "ascii-record-desktop", "scrcpy",
    "scrcpy-server", "start-agents-server", "stop-agents-server", "sync-docker-firewall", "unlisted-tool",
)
KEPT_EXTRAS = ("pnpm", "pnpx", "bun", "bunx", "deno", "uv", "poetry", "sbt", "scala", "scalac", "scaladoc")
RUST_BLOCK = (b'# Rust toolchain (added by install-all-deps.sh)\n'
              b'# RUSTUP_HOME is shared, CARGO_HOME defaults to ~/.cargo per user\n'
              b'export RUSTUP_HOME="/usr/local/rustup"\n'
              b'export PATH="/usr/local/cargo/bin:$PATH"\n\n')


def signature(path):
    if stat.S_ISLNK(path.lstat().st_mode):
        return {"type": "symlink", "link_target": str(path.readlink())}
    return {"type": "file", "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def fixture(workspace, warning=False):
    work = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    root, tools = work / "root", work / "tools"
    tools.mkdir()
    for name in ("usr/bin", "usr/local/bin", "usr/local/sbin", "etc/profile.d", "etc/ciproxy", "etc/apt/apt.conf.d",
                 "root", "home/user", "etc/skel", "usr/share/base-files", "opt/ciproxy-build/definition"):
        (root / name).mkdir(parents=True)
    policy = json.loads((DEFINITION / "boat-developer-tools.json").read_text())
    names = [name for group in policy["commands"] for name in group["names"]]
    targets, commands = {}, {}
    for index, name in enumerate(names):
        if name == "ctest" or name in KEPT_EXTRAS:
            continue
        target = root / "home/user/tool-homes" / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(("provider target " + name).encode())
        targets[target] = signature(target)
        command = root / "usr/local/bin" / name
        if index % 2 == 0:
            command.symlink_to(target)
        else:
            command.write_bytes(("provider command " + name).encode())
        commands[name] = signature(command)
    (root / "usr/local/bin/cargo").unlink()
    (root / "usr/local/bin/cargo").symlink_to("/missing-provider-tool/rustup")
    commands["cargo"] = signature(root / "usr/local/bin/cargo")
    kept = {}
    for name in PROVIDER + KEPT_EXTRAS:
        path = root / "usr/local/bin" / name
        path.write_bytes(("provider entry " + name).encode())
        kept[path] = signature(path)
    for name in ("ascii-idle-reclaim", "provider-extra"):
        path = root / "usr/local/sbin" / name
        path.write_bytes(("provider entry " + name).encode())
        kept[path] = signature(path)
    for name in ("etc/skel/.profile", "etc/skel/.bashrc", "home/user/.profile", "home/user/.bashrc",
                 "etc/profile.d/package-owned.sh", "etc/ciproxy/provider-docker-gid", "usr/bin/node"):
        path = root / name
        path.write_bytes(b"provider state\n")
        kept[path] = signature(path)
    environment = (b'# keep this comment\r\nPATH="/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:'
                   b'/usr/bin:/bin:/usr/local/cargo/bin-extra"\r\nRUSTUP_HOME="/usr/local/rustup"\n'
                   b'ImageOS=ubuntu24\nOTHER="keep  spacing"\r\n# keep non-ASCII bytes: \xff\n')
    expected_environment = environment.replace(b"/usr/local/cargo/bin:", b"").replace(
        b'RUSTUP_HOME="/usr/local/rustup"\n', b"")
    (root / "etc/environment").write_bytes(environment)
    default_bash = (b'# package bash fixture\n\n[ -z "$PS1" ] && return\n'
                    b'# Rust toolchain (added by install-all-deps.sh)\n')
    extra = b"# another local change\n" + RUST_BLOCK if warning else b""
    (root / "etc/bash.bashrc").write_bytes(RUST_BLOCK + default_bash + extra)
    defaults = {".profile": b"# Ubuntu root profile\n", ".bashrc": b"# Ubuntu root bash settings\n"}
    for name, content in defaults.items():
        (root / "usr/share/base-files" / ("dot" + name)).write_bytes(content)
        (root / "root" / name).write_bytes(b"# provider root settings\n")
        kept[root / "root" / name] = signature(root / "root" / name)
    for name in ("rust.sh", "nvm.sh"):
        (root / "etc/profile.d" / name).write_bytes(b"# provider profile script\n")
    kept[root / "etc/profile.d/nvm.sh"] = signature(root / "etc/profile.d/nvm.sh")
    package_state = work / "packages.json"
    package_state.write_text(json.dumps({"md5": hashlib.md5(default_bash).hexdigest(),
                                        "owned": ["/etc/profile.d/rust.sh"] if warning else [],
                                        "installed": {}, "residual": {}, "conffiles": {},
                                        "extra_removals": [], "events": []}))
    query = tools / "dpkg-query"
    query.write_text(f'#!{sys.executable}\nimport json,pathlib,sys\n'
                     f'state=json.loads(pathlib.Path({str(package_state)!r}).read_text())\n'
                     'if sys.argv[1] == "-S":\n'
                     '    owned=sys.argv[-1] in state["owned"]\n'
                     '    print("fixture-package: "+sys.argv[-1]) if owned else None\n'
                     '    raise SystemExit(0 if owned else 1)\n'
                     'if "-f=${Conffiles}" in sys.argv:\n'
                     '    print(" /etc/bash.bashrc "+state["md5"])\n'
                     'else:\n'
                     '    for name,version in state["installed"].items():\n'
                     '        print(name+"\\t"+version+"\\tinstalled")\n'
                     '    for name,version in state["residual"].items():\n'
                     '        print(name+"\\t"+version+"\\tconfig-files")\n')
    query.chmod(0o755)
    definition = root / "opt/ciproxy-build/definition"
    for name in ("boat-build-bootstrap.sh", "boat-build-developer.py", "boat-developer-tools.json"):
        shutil.copy2(DEFINITION / name, definition / name)
    bootstrap = definition / "boat-build-bootstrap.sh"
    bootstrap.write_text(bootstrap.read_text().replace("/etc/", str(root / "etc") + "/").replace(
        "/opt/ciproxy-build", str(root / "opt/ciproxy-build")))
    (root / "opt/ciproxy-build/provider-permissions.json").write_text("{}\n")
    python = tools / "python3"
    python.write_text(f'#!/bin/sh\nexec {sys.executable} "$@" --root "{root}"\n')
    python.chmod(0o755)
    apt = tools / "apt-get"
    apt.write_text(f'#!{sys.executable}\nimport json,pathlib,sys\nroot=pathlib.Path({str(root)!r})\n'
                   f'record=root/{str(RECORD)!r}\n'
                   'if "remove" in sys.argv or "purge" in sys.argv:\n'
                   '    action="purge" if "purge" in sys.argv else "remove"\n'
                   '    assert "--no-auto-remove" in sys.argv\n'
                   '    assert not (root/"usr/local/bin/cmake").is_symlink()\n'
                   '    assert (root/"etc/bash.bashrc").read_bytes().startswith(b"# package bash fixture")\n'
                   f'    path=pathlib.Path({str(package_state)!r})\n'
                   '    state=json.loads(path.read_text())\n'
                   '    names=sys.argv[sys.argv.index("--")+1:]\n'
                   '    simulation="--simulate" in sys.argv\n'
                   '    state["events"].append({"simulation":simulation,"packages":names,"action":action})\n'
                   '    if simulation:\n'
                   '        extra=state["extra_removals"] if "dotnet-host-8.0" in names else []\n'
                   '        for name in names+extra: print(("Purg " if action=="purge" else "Remv ")+name)\n'
                   '    else:\n'
                   '        for name in names:\n'
                   '            version=state["installed"].pop(name,state["residual"].get(name))\n'
                   '            if action=="purge":\n'
                   '                state["residual"].pop(name,None)\n'
                   '                for relative in state["conffiles"].get(name,[]): (root/relative).unlink(missing_ok=True)\n'
                   '            else: state["residual"][name]=version\n'
                   '    path.write_text(json.dumps(state))\n'
                   '    raise SystemExit(0)\n'
                   'if sys.argv[1] == "update" and record.exists():\n'
                   '    print("Cleanup must follow the package index update.")\n'
                   '    raise SystemExit(24)\n'
                   'if sys.argv[1] == "install" and not record.exists():\n'
                   '    print("Developer cleanup must finish before package installation.")\n'
                   '    raise SystemExit(23)\n')
    apt.chmod(0o755)
    env = {**os.environ, "PATH": str(tools) + os.pathsep + os.environ["PATH"]}
    return types.SimpleNamespace(root=root, env=env, policy=policy, commands=commands, targets=targets,
                                 kept=kept, environment=expected_environment, bash=default_bash + extra,
                                 bootstrap=bootstrap, package_state=package_state)


def run_cleanup(case, sample, bootstrap=False):
    command = (["bash", str(sample.bootstrap)] if bootstrap else
               [sys.executable, str(DEFINITION / "boat-build-developer.py"), "--root", str(sample.root)])
    result = subprocess.run(command, env=sample.env, capture_output=True, text=True)
    case.assertEqual(result.returncode, 0, result.stdout + result.stderr)
    record = sample.root / RECORD
    case.assertTrue(record.is_file(), "Cleanup must write its first-run record.")
    return json.loads(record.read_text())


def check_developer(case, workspace):
    sample = fixture(workspace)
    report = run_cleanup(case, sample, bootstrap=True)
    commands = {item["name"]: item for item in report["commands"]}
    names = [name for group in sample.policy["commands"] for name in group["names"]]
    case.assertEqual(set(commands), set(names))
    case.assertEqual(commands["ctest"]["status"], "absent")
    for name, before in sample.commands.items():
        case.assertFalse((sample.root / "usr/local/bin" / name).is_symlink(), name)
        case.assertFalse((sample.root / "usr/local/bin" / name).exists(), name)
        case.assertEqual(commands[name]["status"], "removed", name)
        case.assertEqual({key: commands[name][key] for key in before}, before, name)
    for path, before in {**sample.targets, **sample.kept}.items():
        case.assertTrue(path.is_symlink() or path.is_file(), "Kept entry must stay: " + path.name)
        case.assertEqual(signature(path), before, path.name)
    case.assertEqual(report["kept_entries"], {"/usr/local/bin": sorted(PROVIDER + KEPT_EXTRAS),
                                           "/usr/local/sbin": ["ascii-idle-reclaim", "provider-extra"]})
    case.assertEqual(hashlib.sha256((sample.root / "etc/environment").read_bytes()).hexdigest(),
                     hashlib.sha256(sample.environment).hexdigest(), "Only listed shell settings may change.")
    bash = (sample.root / "etc/bash.bashrc").read_bytes()
    case.assertEqual(bash, sample.bash, "Cleanup must restore the exact package bytes.")
    case.assertIn(b"# package bash fixture\n\n", bash, "The package's empty line must stay.")
    settings = {item["path"]: item for item in report["settings"]}
    case.assertEqual(settings["/etc/environment"]["status"], "changed")
    case.assertTrue(settings["/etc/bash.bashrc"]["verified_md5"])
    case.assertEqual(report["warnings"], [])
    case.assertFalse((sample.root / "etc/profile.d/rust.sh").exists())
    case.assertEqual(settings["/etc/profile.d/rust.sh"]["status"], "removed")
    for path in ("/etc/profile.d/nvm.sh", "/root/.profile", "/root/.bashrc"):
        case.assertEqual(settings[path]["status"], "kept")
    extras = {item["name"]: item for item in report["kept_extras"]}
    case.assertEqual(set(extras), set(KEPT_EXTRAS))
    for name in KEPT_EXTRAS:
        case.assertEqual(extras[name]["status"], "kept", name)
    (sample.root / "usr/local/bin/cmake").write_text("new shared CMake\n")
    before = {path: (signature(path), path.lstat().st_mtime_ns) for path in sample.root.rglob("*")
              if path.is_symlink() or path.is_file()}
    run_cleanup(case, sample)
    after = {path: (signature(path), path.lstat().st_mtime_ns) for path in sample.root.rglob("*")
             if path.is_symlink() or path.is_file()}
    case.assertEqual(after, before, "A repeat must preserve every file and the first record.")


def check_developer_warnings(case, workspace):
    sample = fixture(workspace, warning=True)
    owned = sample.root / "etc/profile.d/rust.sh"
    before = signature(owned)
    report = run_cleanup(case, sample)
    settings = {item["path"]: item for item in report["settings"]}
    case.assertEqual(settings["/etc/profile.d/rust.sh"]["status"], "kept-package-owned")
    case.assertEqual(signature(owned), before)
    for path, signature_before in sample.kept.items():
        case.assertTrue(path.is_symlink() or path.is_file(), "Kept entry must stay: " + path.name)
        case.assertEqual(signature(path), signature_before, path.name)
    case.assertFalse(settings["/etc/bash.bashrc"]["verified_md5"])
    case.assertEqual((sample.root / "etc/bash.bashrc").read_bytes(), sample.bash)
    case.assertEqual((sample.root / "etc/bash.bashrc").read_bytes().count(RUST_BLOCK), 1,
                     "Cleanup must remove only the first exact block.")
    case.assertEqual(report["warnings"], ["/etc/bash.bashrc does not match the installed package's conffile md5."])


def check_developer_inventory(case, workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    packages = types.SimpleNamespace(returncode=0, stdout="fixture-package\t1\n")
    inventory = root / "reports/installed-inventory.json"
    expected = {"commands": [{"name": "cmake", "status": "removed"}], "warnings": []}
    with patch("boat_build_report.subprocess.run", return_value=packages):
        collect(root, root / "imagegeneration", root / "restart-services")
        case.assertIn("provider_developer_environment", json.loads(inventory.read_text()))
        case.assertIsNone(json.loads(inventory.read_text())["provider_developer_environment"])
        (root / RECORD.name).write_text(json.dumps(expected))
        collect(root, root / "imagegeneration", root / "restart-services")
    case.assertEqual(json.loads(inventory.read_text())["provider_developer_environment"], expected)


def check_developer_packages(case, workspace):
    sample = fixture(workspace)
    installed = {"cmake": "3.28.3-1build7", "cmake-data": "3.28.3-1build7",
                 "dotnet-host-8.0": "8.0.11-0ubuntu1", "dotnet-sdk-8.0": "8.0.110-0ubuntu1",
                 "aspnetcore-runtime-8.0": "8.0.11-0ubuntu1", "netstandard-targeting-pack-2.1-8.0": "8.0.11-0ubuntu1",
                 "php8.4-cli": "8.4.16-1", "php8.4-common": "8.4.16-1", "php8.3-cli": "8.3.14-1",
                 "nodejs": "20.19.0", "ninja-build": "1.11.1-2", "openjdk-11-jre-headless": "11.0.29",
                 "provider-controller": "1"}
    state = json.loads(sample.package_state.read_text())
    state.update(installed=installed, extra_removals=["provider-controller"])
    sample.package_state.write_text(json.dumps(state))
    report = run_cleanup(case, sample)
    case.assertEqual([(item["pinned_tool"], item["status"]) for item in report.get("packages", [])],
                     [("CMake 3.31.6", "removed"), (".NET", "kept"), ("Maven", "absent"), ("PHP 8.3", "removed")])
    groups = {item["pinned_tool"]: item for item in report["packages"]}
    removed = {"cmake", "cmake-data", "php8.4-cli", "php8.4-common"}
    for group in groups.values():
        case.assertEqual(group.get("action"), "purge")
        for item in group["packages"]:
            case.assertEqual(item["version"], installed[item["name"]])
            case.assertEqual(item["status"], "removed" if item["name"] in removed else "kept")
    case.assertEqual(groups["Maven"]["packages"], [])
    case.assertEqual(groups[".NET"]["extra_removals"], ["provider-controller"])
    case.assertEqual({item["name"] for item in groups[".NET"]["packages"]},
                     {"dotnet-host-8.0", "dotnet-sdk-8.0", "aspnetcore-runtime-8.0", "netstandard-targeting-pack-2.1-8.0"})
    case.assertEqual(report["warnings"], ["Kept .NET packages because apt would also remove: provider-controller."])
    final = json.loads(sample.package_state.read_text())
    case.assertEqual(final["installed"], {name: version for name, version in installed.items() if name not in removed})
    removals = [set(event["packages"]) for event in final["events"] if not event["simulation"]]
    case.assertEqual(removals, [{"cmake", "cmake-data"}, {"php8.4-cli", "php8.4-common"}])
    case.assertEqual([event["simulation"] for event in final["events"]], [True, False, True, True, False])
    case.assertEqual({event["action"] for event in final["events"]}, {"purge"})
