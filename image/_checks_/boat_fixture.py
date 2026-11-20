# SPDX-License-Identifier: Apache-2.0
"""Run the native startup script in a private filesystem with a service spy."""

import configparser
import pathlib
import re
import shutil
import subprocess


def copy_program(source, root, destination=None):
    source = pathlib.Path(source).resolve()
    target = root / (destination or str(source)).lstrip("/")
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)
    target.chmod(0o755)
    if source.read_bytes().startswith(b"#!"):
        interpreter = source.read_text().splitlines()[0][2:].split()[0]
        copy_program(interpreter, root)
        return
    libraries = subprocess.check_output(["ldd", str(source)], text=True)
    for library in re.findall(r"(/[^\s()]+)", libraries):
        target = root / library.lstrip("/")
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(library, target)
        target.chmod(0o755)


def prepare_root(root, guest):
    image = pathlib.Path(__file__).resolve().parents[1]
    for relative in ["usr/bin", "image", "dev", "state", "etc/ciproxy", "opt/ciproxy-image"]:
        (root / relative).mkdir(parents=True, exist_ok=True)
    for name in ["bash", "env", "install", "rm", "cp", "cat"]:
        copy_program(shutil.which(name), root, "/usr/bin/" + name)
    copy_program(guest, root, "/usr/local/bin/ciproxy-guest")
    (root / "bin").symlink_to("usr/bin")
    (root / "dev/null").touch()
    shutil.copyfile(image / "boat-run.sh", root / "image/boat-run.sh")
    startup = image / "ciproxy-boat-start.service"
    service = configparser.ConfigParser(interpolation=None)
    service.read(startup)
    shutil.copyfile(image / "boat-start.sh", root / "opt/ciproxy-image/boat-start.sh")
    (root / "opt/ciproxy-image/boat-start.sh").chmod(0o755)
    (root / "etc/ciproxy/boat-restart-services").write_text(
        "containerd.service\ndocker.service\nexample.service\nuser@1000.service\n")
    (root / "etc/passwd").write_text("root:x:0:0:Root:/root:/bin/bash\n")
    (root / "etc/group").write_text("root:x:0:\n")
    systemctl = root / "usr/bin/systemctl"
    systemctl.write_text('''#!/usr/bin/bash
printf '%s\\n' "$*" >> /state/events
if [[ "$*" == 'start --no-block ciproxy-boat-start.service' ]]; then
    export CIPROXY_FIXTURE_SYSTEM_SCOPE=1
    @START_COMMAND@
    exit "$?"
fi
if [[ "$*" == 'try-restart user@1000.service' && ${CIPROXY_FIXTURE_SYSTEM_SCOPE:-} != 1 ]]; then
    exit 75
fi
if [[ "$*" == 'start ciproxy-boat-guest.service' ]]; then
    if [[ -e /run/ciproxy/registration.json ]]; then
        cp -p /run/ciproxy/registration.json /state/delivered-registration.json
    else
        /usr/local/bin/ciproxy-guest --image-id fixture-image boat \\
            --registration-file /run/ciproxy/registration.json > /state/guest.log 2>&1
        printf '%s' "$?" > /state/guest-exit
    fi
fi
exit 0
'''.replace("@START_COMMAND@", service["Service"]["ExecStart"]))
    systemctl.chmod(0o755)
    python = root / "usr/bin/python3"
    python.write_text("#!/usr/bin/bash\nexit 0\n")
    python.chmod(0o755)
