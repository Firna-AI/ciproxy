//! Runner wrapper integration through real hooks and environment files.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tempfile::TempDir;

#[test]
fn wrapper_preserves_the_existing_hook_and_publishes_container_environment() {
    let directory = TempDir::new().unwrap();
    let wrapper = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../image/runner-patch/run-with-cache.sh");
    let prior = directory.path().join("existing hook.sh");
    let runner = directory.path().join("runner.sh");
    let environment = directory.path().join("workflow-environment");
    fs::write(&runner, "status=0\nbash -e \"$ACTIONS_RUNNER_HOOK_JOB_STARTED\" || status=$?\nprintf '%s' \"$ACTIONS_RUNTIME_TOKEN\"\nexit \"$status\"\n").unwrap();
    for status in [0, 7] {
        fs::write(&environment, "").unwrap();
        fs::write(
            &prior,
            format!("false | true\nprintf 'PRIOR=kept\\n' >> \"$GITHUB_ENV\"\nexit {status}\n"),
        )
        .unwrap();
        let output = Command::new("bash")
            .arg(&wrapper)
            .arg("http://172.17.0.1:8230")
            .arg("bash")
            .arg(&runner)
            .env("ACTIONS_RUNNER_HOOK_JOB_STARTED", &prior)
            .env("CIPROXY_RUNNER_ROOT", directory.path())
            .env("GITHUB_ENV", &environment)
            .env("ACTIONS_RUNTIME_TOKEN", "original-runtime-token")
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(status),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"original-runtime-token");
        let contents = fs::read_to_string(&environment).unwrap();
        let values: BTreeMap<_, _> = contents
            .lines()
            .map(|line| line.split_once('=').unwrap())
            .collect();
        assert_eq!(values.get("PRIOR"), Some(&"kept"));
        assert_eq!(
            values.get("ACTIONS_CACHE_URL"),
            Some(&"http://172.17.0.1:8230/")
        );
        assert_eq!(
            values.get("ACTIONS_RESULTS_URL"),
            Some(&"http://172.17.0.1:8230/")
        );
        assert_eq!(values.get("ACTIONS_CACHE_SERVICE_V2"), Some(&"true"));
        let context_environment = Command::new("env")
            .arg("-i")
            .args(contents.lines())
            .arg("/usr/bin/env")
            .output()
            .unwrap();
        assert!(context_environment.status.success());
        assert!(
            String::from_utf8_lossy(&context_environment.stdout)
                .contains("ACTIONS_RESULTS_URL=http://172.17.0.1:8230/")
        );
    }
}
