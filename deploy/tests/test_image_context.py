"""Source reuse and the credentialless builder use the same tracked context."""

from pathlib import Path

from deploy.tests.image_support import ImageCase


class ImageContextTests(ImageCase):
    def test_source_hash_tracks_all_build_inputs_and_reuses_excluded_docs(self):
        previous = self.image_hash()
        for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
                     "crates/ciproxy-control-plane/Dockerfile", "crates/runtime/Cargo.toml",
                     "crates/runtime/build.rs", "crates/runtime/src/main.rs",
                     "crates/runtime/migrations/001/up.sql", "xtask/src/main.rs",
                     ".dockerignore", "deploy/scripts/install-image-tools.sh"):
            with self.subTest(input=name):
                path = self.root / name
                path.write_text(path.read_text() + "\n")
                self.commit()
                current = self.image_hash()
                self.assertNotEqual(current, previous, name)
                previous = current
        for name in ("crates/runtime/README.md", "docs/guide.md",
                     "crates/runtime/.env", "crates/runtime/private/operator.key"):
            path = self.root / name
            path.write_text(path.read_text() + "excluded\n")
        self.commit()
        self.assertEqual(self.image_hash(), previous)
        executable = self.root / "crates/runtime/build.rs"
        executable.chmod(0o755)
        self.commit()
        self.assertNotEqual(self.image_hash(), previous)

    def test_builder_uses_hashed_context_and_newly_allowed_input_patterns(self):
        (self.root / "crates/runtime/template.bin").write_text("runtime template\n")
        with (self.root / ".dockerignore").open("a") as output:
            output.write("\n!crates/**/*.bin\n")
        self.commit()
        source_hash = self.image_hash()
        result = self.image("build", "--source", ".", "--revision", self.git("rev-parse", "HEAD").strip(),
                            "--source-hash", source_hash, "--archive", ".context/image.tar")
        self.assertEqual(result.returncode, 0, result.stderr)
        call = self.calls()[0]
        self.assertEqual(call["context"]["crates/runtime/template.bin"], "runtime template\n")
        self.assertEqual(call["context"]["crates/runtime/src/main.rs"], "fn main() {}\n")
        self.assertNotIn("crates/runtime/README.md", call["context"])
        self.assertNotIn("crates/runtime/.env", call["context"])
        self.assertNotIn("crates/runtime/private/operator.key", call["context"])
        self.assertNotIn("deploy/scripts/control_plane_image.py", call["context"])
        self.assertIn("platform=linux/amd64", call["args"])
        self.assertTrue(call["docker_config_empty"])
        self.assertEqual(call["config_mode"], 0o700)
        self.assertFalse(Path(call["config"]).exists())
        self.assertEqual((self.root / ".context/image.tar").read_bytes(), b"built-image")
        (self.root / "crates/runtime/template.bin").write_text("changed template\n")
        self.commit()
        self.assertNotEqual(self.image_hash(), source_hash)
