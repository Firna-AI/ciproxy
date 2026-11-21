"""Private temporary repositories and command doubles for image CLI tests."""

import json
import os
from pathlib import Path
import shutil
import sys
import tempfile

from deploy.tests.promotion_support import PromotionCase, ROOT


REPOSITORY = "us-central1-docker.pkg.dev/ciproxy/ciproxy-shared-runtime/control-plane"
IMAGE = REPOSITORY + "@sha256:" + "a" * 64

TOOLS = r'''
import json
import os
from pathlib import Path
import stat
import sys

tool, args = Path(sys.argv[0]).name, sys.argv[1:]
record = {"tool": tool, "args": args, "config": os.environ.get("DOCKER_CONFIG", "")}
config = Path(record["config"]) if record["config"] else None
if config:
    record["config_mode"] = stat.S_IMODE(config.stat().st_mode)
if tool == "crane" and args[:2] == ["auth", "login"]:
    record["token_on_stdin"] = sys.stdin.read() == "fixture-token-not-a-secret"
    (config / "config.json").write_text('{"auths":{"fixture":"private"}}')
if tool == "buildctl":
    context = Path(next(arg.split("=", 1)[1] for arg in args if arg.startswith("context=")))
    record["context"] = {str(path.relative_to(context)): path.read_text()
                         for path in context.rglob("*") if path.is_file()}
    record["docker_config_empty"] = list(config.iterdir()) == []
    output = next(arg for arg in args if arg.startswith("type=docker,"))
    Path(output.split("dest=", 1)[1]).write_bytes(b"built-image")
with open(os.environ["IMAGE_COMMAND_LOG"], "a") as output:
    output.write(json.dumps(record) + "\n")
if os.environ.get("IMAGE_FAIL_COMMAND") == tool + " " + " ".join(args[:2]):
    print("fixture-token-not-a-secret private auth failure", file=sys.stderr)
    sys.exit(1)
if tool == "gcloud" and args[:2] == ["artifacts", "repositories"]:
    if os.environ.get("IMAGE_REPOSITORY_ERROR"):
        print(os.environ["IMAGE_REPOSITORY_ERROR"], file=sys.stderr)
        sys.exit(1)
    print("projects/ciproxy/locations/us-central1/repositories/ciproxy-shared-runtime")
elif tool == "gcloud":
    print("fixture-token-not-a-secret")
elif tool == "crane":
    state_path = Path(os.environ["IMAGE_REGISTRY_STATE"])
    state = json.loads(state_path.read_text())
    if args[0] == "digest":
        if args[1] == "--tarball":
            print("sha256:" + "a" * 64)
        else:
            digest = args[1].split("@", 1)[1] if "@" in args[1] else state.get(args[1])
            error = os.environ.get("IMAGE_LOOKUP_ERROR")
            if error or not digest:
                print(error or "GET: MANIFEST_UNKNOWN: manifest unknown", file=sys.stderr)
                sys.exit(1)
            print(digest)
    elif args[0] == "tag":
        state[args[1].split("@", 1)[0] + ":" + args[2]] = args[1].split("@", 1)[1]
        state_path.write_text(json.dumps(state))
'''


class ImageCase(PromotionCase):
    def setUp(self):
        scratch = ROOT / ".context/control-plane-image/tests"
        scratch.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        shutil.copytree(ROOT / "deploy/scripts", self.root / "deploy/scripts")
        self.env = dict(os.environ)
        self.env.pop("GITHUB_OUTPUT", None)
        self.env.pop("GITHUB_EVENT_PATH", None)
        files = {
            ".dockerignore": (ROOT / ".dockerignore").read_text(),
            "Cargo.toml": "[workspace]\n", "Cargo.lock": "version = 4\n",
            "rust-toolchain.toml": '[toolchain]\nchannel = "1.99.1"\n',
            "crates/ciproxy-control-plane/Dockerfile": (ROOT / "crates/ciproxy-control-plane/Dockerfile").read_text(),
            "crates/runtime/Cargo.toml": '[package]\nname = "runtime"\n',
            "crates/runtime/build.rs": "fn main() {}\n",
            "crates/runtime/src/main.rs": "fn main() {}\n",
            "crates/runtime/migrations/001/up.sql": "SELECT 1;\n",
            "xtask/Cargo.toml": '[package]\nname = "xtask"\n',
            "xtask/src/main.rs": "fn main() {}\n",
            "crates/runtime/README.md": "Runtime documentation.\n",
            "crates/runtime/private/operator.key": "fixture-private-input\n",
            "crates/runtime/.env": "FIXTURE=private\n",
            "docs/guide.md": "Public documentation.\n",
            ".github/workflows/deploy-control-plane-image.yml": "name: Fixture protected build\n",
            ".github/workflows/deploy-main-preview.yml": "name: Fixture caller\n",
        }
        for name, content in files.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        self.commit()
        self.source = self.git("rev-parse", "HEAD").strip()
        tools = self.root / ".context/bin"
        tools.mkdir(parents=True)
        self.log = self.root / ".context/image-commands.jsonl"
        self.log.write_text("")
        self.state = self.root / ".context/registry.json"
        self.state.write_text("{}")
        for name in ("gcloud", "crane", "buildctl"):
            path = tools / name
            path.write_text(f"#!{sys.executable}\n" + TOOLS)
            path.chmod(0o755)
        self.env.update(PATH=str(tools) + os.pathsep + self.env["PATH"],
                        IMAGE_COMMAND_LOG=str(self.log), IMAGE_REGISTRY_STATE=str(self.state))

    def commit(self):
        if not (self.root / ".git").exists():
            self.git("init", "-q")
            self.git("config", "user.email", "fixture@example.invalid")
            self.git("config", "user.name", "Fixture")
            (self.root / ".gitignore").write_text(".context/\n__pycache__/\n")
        self.git("add", "-A")
        self.git("commit", "-qm", "fixture")

    def image(self, *arguments):
        return self.cli("control_plane_image.py", *arguments)

    def image_hash(self):
        result = self.image("inputs", "--source", ".", "--revision", self.git("rev-parse", "HEAD").strip())
        self.assertEqual(result.returncode, 0, result.stderr)
        return dict(line.split("=", 1) for line in result.stdout.splitlines())["source_hash"]

    def calls(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()]
