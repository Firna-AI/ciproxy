//! launchd files and `launchctl` arguments for the agent and its VM helpers.

use super::paths::{
    AGENT_LABEL, AGENT_PLIST, BINARY, ENVIRONMENT, LOG, local_id_of_label, vm_label, vm_plist,
};

/// How launchd restarts a job.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum KeepAlive {
    /// Restart after a failed exit only, like systemd `Restart=on-failure`.
    /// A removed machine exits with 0, so launchd does not start it again.
    OnFailure,
    /// Run once. A VM helper ends with its VM.
    Never,
}

struct Job<'a> {
    label: &'a str,
    arguments: &'a [&'a str],
    keep_alive: KeepAlive,
    /// Seconds between SIGTERM and SIGKILL.
    exit_timeout: u32,
}

/// The agent daemon in `/Library/LaunchDaemons`. It starts at boot.
///
/// launchd has no environment file, so `/bin/sh` loads the settings file,
/// as systemd's `EnvironmentFile` does on Linux. The shell runs as root, so
/// it loads the file only when root owns it and no group or other user can
/// write it. Then the shell replaces itself with the agent.
pub(crate) fn agent_plist() -> String {
    let start = format!(
        "if [ -f {ENVIRONMENT} ] && [ -O {ENVIRONMENT} ] && [ -z \"$(find {ENVIRONMENT} -perm +022)\" ]; then set -a; . {ENVIRONMENT}; set +a; fi; exec {BINARY} run"
    );
    render(&Job {
        label: AGENT_LABEL,
        arguments: &["/bin/sh", "-c", &start],
        keep_alive: KeepAlive::OnFailure,
        exit_timeout: 30,
    })
}

/// One VM helper. SIGTERM stops its VM at once, as a `systemctl stop` of
/// a Firecracker unit does.
pub(crate) fn vm_helper_plist(local_id: &str) -> String {
    let label = vm_label(local_id);
    render(&Job {
        label: &label,
        arguments: &[BINARY, "vm-run", local_id],
        keep_alive: KeepAlive::Never,
        exit_timeout: 30,
    })
}

fn render(job: &Job<'_>) -> String {
    let mut text = String::from(concat!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
        "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" ",
        "\"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n",
        "<plist version=\"1.0\">\n<dict>\n",
    ));
    entry(&mut text, "Label", &string(job.label));
    let arguments: String = job.arguments.iter().map(|value| string(value)).collect();
    entry(
        &mut text,
        "ProgramArguments",
        &format!("<array>{arguments}</array>"),
    );
    entry(&mut text, "RunAtLoad", "<true/>");
    match job.keep_alive {
        KeepAlive::OnFailure => {
            entry(
                &mut text,
                "KeepAlive",
                "<dict><key>SuccessfulExit</key><false/></dict>",
            );
            entry(&mut text, "ThrottleInterval", "<integer>5</integer>");
        }
        KeepAlive::Never => entry(&mut text, "KeepAlive", "<false/>"),
    }
    entry(
        &mut text,
        "ExitTimeOut",
        &format!("<integer>{}</integer>", job.exit_timeout),
    );
    entry(&mut text, "StandardOutPath", &string(LOG));
    entry(&mut text, "StandardErrorPath", &string(LOG));
    text.push_str("</dict>\n</plist>\n");
    text
}

fn entry(text: &mut String, key: &str, value: &str) {
    text.push_str(&format!("  <key>{key}</key>\n  {value}\n"));
}

fn string(value: &str) -> String {
    let escaped = value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!("<string>{escaped}</string>")
}

/// Rotate the log at 10 MB and keep 5 old files, with no signal. The agent
/// and its VM helpers open the log again after a rotation. launchd keeps
/// its own descriptor only for early output, such as a panic.
pub(crate) fn newsyslog_conf() -> String {
    format!("# ciproxy agent logs\n{LOG}\troot:wheel\t600\t5\t10240\t*\tNJ\n")
}

/// Read the local IDs from `launchctl list`: PID, status, and label.
pub(crate) fn loaded_ids(list: &str) -> Vec<String> {
    list.lines()
        .filter_map(|line| line.split('\t').nth(2))
        .filter_map(local_id_of_label)
        .map(str::to_owned)
        .collect()
}

/// `launchctl` arguments. Each job is in the system domain.
pub(crate) mod launchctl {
    use super::{AGENT_LABEL, AGENT_PLIST, vm_label, vm_plist};

    fn target(label: &str) -> String {
        format!("system/{label}")
    }

    pub(crate) fn bootstrap_agent() -> Vec<String> {
        vec!["bootstrap".into(), "system".into(), AGENT_PLIST.into()]
    }

    pub(crate) fn enable_agent() -> Vec<String> {
        vec!["enable".into(), target(AGENT_LABEL)]
    }

    /// Restart the running agent, for a reinstall or an update.
    pub(crate) fn restart_agent() -> Vec<String> {
        vec!["kickstart".into(), "-k".into(), target(AGENT_LABEL)]
    }

    /// Stop the agent and unload it. VM helpers are other jobs and keep running.
    pub(crate) fn bootout_agent() -> Vec<String> {
        vec!["bootout".into(), target(AGENT_LABEL)]
    }

    pub(crate) fn print_agent() -> Vec<String> {
        vec!["print".into(), target(AGENT_LABEL)]
    }

    pub(crate) fn bootstrap_vm(local_id: &str) -> Vec<String> {
        vec![
            "bootstrap".into(),
            "system".into(),
            vm_plist(local_id).to_string_lossy().into_owned(),
        ]
    }

    /// Unload a helper. launchd sends SIGTERM, so the helper stops its VM.
    pub(crate) fn bootout_vm(local_id: &str) -> Vec<String> {
        vec!["bootout".into(), target(&vm_label(local_id))]
    }

    pub(crate) fn print_vm(local_id: &str) -> Vec<String> {
        vec!["print".into(), target(&vm_label(local_id))]
    }

    /// List the loaded system jobs, to find helpers with no record.
    pub(crate) fn list() -> Vec<String> {
        vec!["list".into()]
    }
}
