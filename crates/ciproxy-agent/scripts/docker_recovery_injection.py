"""Temporary systemd PATH injection for a disposable KVM smoke host."""

from contextlib import contextmanager
import json
from pathlib import Path
import shutil

from own_smoke import Failure


@contextmanager
def mount_failure(smoke, base):
    real = shutil.which("dumpe2fs")
    if not real:
        raise Failure("Install e2fsprogs before the guest mount check")
    directory = (smoke.evidence / "fault").resolve()
    directory.mkdir(mode=0o700)
    wrapper = directory / "dumpe2fs"
    shutil.copyfile(Path(__file__).with_name("docker-recovery-dumpe2fs.py"), wrapper)
    wrapper.chmod(0o700)
    receipt = directory / "receipt.json"
    (directory / "fault.json").write_text(json.dumps({
        "real": real, "base": str(base), "state": str(smoke.state_file.resolve()),
        "area": str(base.parent.parent.parent), "receipt": str(receipt),
    }))
    pid = smoke.command("systemctl", "show", "--property=MainPID", "--value",
                        "ciproxy-agent.service", capture=True)
    environment = Path("/proc").joinpath(pid, "environ").read_bytes().split(b"\0")
    original = next(value[5:].decode() for value in environment if value.startswith(b"PATH="))
    path = str(directory) + ":" + original
    if any(character in path for character in ["\n", "\r", '"', "\\", "%"]):
        raise Failure("Use a host path without systemd quoting characters")
    parent = Path("/run/systemd/system/ciproxy-agent.service.d")
    parent.mkdir(parents=True, exist_ok=True)
    override = parent / ("docker-recovery-" + smoke.tag + ".conf")
    try:
        with override.open("x") as output:
            output.write('[Service]\nExecStart=\nExecStart=/usr/bin/env "PATH=' + path
                         + '" /usr/local/bin/ciproxy-agent run\n')
        smoke.command("systemctl", "daemon-reload")
        smoke.command("systemctl", "restart", "ciproxy-agent.service")
        yield receipt
    finally:
        override.unlink(missing_ok=True)
        smoke.command("systemctl", "daemon-reload")
        smoke.command("systemctl", "restart", "ciproxy-agent.service")
