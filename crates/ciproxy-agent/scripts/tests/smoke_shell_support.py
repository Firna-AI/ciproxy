"""Stand-in host commands for the real shell smoke entries."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from smoke_image_support import index


SCRIPTS = Path(__file__).resolve().parents[1]
EVIDENCE = SCRIPTS.parents[2] / ".context/ciproxy-mvp/m27g"

HOST_COMMANDS = r'''
source "$SMOKE_SCRIPTS/smoke-host.sh"
function [() {
  if [[ "${2-}" == /dev/kvm ]]; then return 0; fi
  builtin [ "$@"
}
uname() { if [[ "$1" == -s ]]; then echo Linux; else echo x86_64; fi; }
id() { echo 0; }
systemctl() { return 0; }
journalctl() { return 0; }
gh() {
  case "$1 $2" in
    'auth status') return 0 ;;
    'workflow run') printf 'dispatch\n' >>"$SMOKE_EVENTS"; return 19 ;;
    'run cancel') printf '%s\n' "$*" >>"$SMOKE_EVENTS"; return "$CANCEL_EXIT" ;;
    *) return 93 ;;
  esac
}
python3() { command "$SMOKE_PYTHON" -B "$SMOKE_WRAPPER" "$@"; }
sleep() { printf 'warm-before-image\n' >>"$SMOKE_EVENTS"; return 29; }
'''

PYTHON_CLOCK = '''
import json
import os
from pathlib import Path
import runpy
import sys
import time

now = 0
def sleep(seconds):
    global now
    now += seconds
    if os.environ.get("STAY_COLD"):
        return
    Path(os.environ["CIPROXY_SMOKE_IMAGE_INDEX"]).write_text(json.dumps({
        "newest": "wanted", "available": [
            {"image_id": "wanted", "root_disk_bytes": 60263759872}]}))
    Path(os.environ["CIPROXY_SMOKE_STATE"]).write_text(json.dumps({
        "identity": {}, "mode": "Active", "runners": [{"local_id": "warm-vm",
        "report": {"runner_id": "warm-runner", "state": {"state": "warm"}}}]}))
    with Path(os.environ["SMOKE_EVENTS"]).open("a") as output:
        output.write("published\\n")

time.monotonic = lambda: now
time.sleep = sleep
sys.argv = [arg for arg in sys.argv[1:] if arg != "-B"]
runpy.run_path(sys.argv[0], run_name="__main__")
'''


class ShellHost:
    def __enter__(self):
        EVIDENCE.mkdir(parents=True, exist_ok=True)
        self.directory = tempfile.TemporaryDirectory(prefix="shell-", dir=EVIDENCE)
        self.root = Path(self.directory.name)
        self.events = self.root / "events"
        self.events.touch()
        self.image = self.root / "index.json"
        self.image.write_text(index(None))
        self.state = self.root / "state.json"
        self.state.write_text(json.dumps({"identity": {}, "mode": "Active", "runners": []}))
        credentials = self.root / "credentials.json"
        credentials.write_text(json.dumps({"github_token": "public-test-value"}))
        wrapper = self.root / "clock.py"
        wrapper.write_text(PYTHON_CLOCK)
        self.environment = dict(os.environ, CIPROXY_SMOKE_CREDENTIALS=str(credentials),
            CIPROXY_SMOKE_REPO="owner/private-smoke", CIPROXY_SMOKE_REF="main",
            CIPROXY_SMOKE_STATE=str(self.state), CIPROXY_SMOKE_EVIDENCE=str(self.root),
            CIPROXY_SMOKE_IMAGE_INDEX=str(self.image), CIPROXY_SMOKE_IMAGE_RELEASE="wanted",
            SMOKE_EVENTS=str(self.events), SMOKE_SCRIPTS=str(SCRIPTS),
            SMOKE_WRAPPER=str(wrapper), SMOKE_PYTHON=sys.executable, CANCEL_EXIT="0")
        self.environment.pop("CIPROXY_SMOKE_IMAGE_TIMEOUT_SECONDS", None)
        return self

    def __exit__(self, *args):
        self.directory.cleanup()

    def run(self, script, **extra):
        return subprocess.run(["bash", "-c", HOST_COMMANDS + script],
            cwd=SCRIPTS.parents[2], env=dict(self.environment, **extra),
            capture_output=True, text=True, timeout=10, check=False)

    def observations(self):
        return self.events.read_text().splitlines()
