//! A separate fake GitHub process whose wire contract comes from upstream.

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

pub struct Github {
    child: Child,
    pub origin: String,
}

impl Github {
    pub fn start() -> Self {
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../image/runner-patch/smoke/fake-github.py");
        let mut child = Command::new("python3")
            .arg(script)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut origin = String::new();
        BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut origin)
            .unwrap();
        assert!(
            origin.starts_with("http://127.0.0.1:"),
            "fixture did not report its listener"
        );
        Self {
            child,
            origin: origin.trim().to_owned(),
        }
    }
}

impl Drop for Github {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
