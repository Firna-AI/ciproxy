//! Unsupported preparation commands warn without changing the runner lifecycle.

use std::io::{Result, Write};
use std::sync::{Arc, Mutex};

use ciproxy_protocol::guest::{AgentToGuest, JobDecision};
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::runners::RunnerExit;
use tracing::Level;

use crate::boundary::RunnerEvent;
use crate::service::Guest;

use super::support::{
    Observation, channel, configure, configured, exited, hello, runner, supervisor,
};

#[derive(Clone, Default)]
struct Warnings(Arc<Mutex<Vec<u8>>>);

impl Write for Warnings {
    fn write(&mut self, bytes: &[u8]) -> Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn preparation_for_an_inactive_job_warns_and_keeps_the_runner() {
    let warnings = Warnings::default();
    let captured = warnings.clone();
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_max_level(Level::WARN)
        .with_writer(move || captured.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);
    let trace = Arc::new(Mutex::new(Vec::new()));
    let preparation = AgentToGuest::PrepareJob {
        job_id: JobId("inactive-job".into()),
        decision: JobDecision::GitHubOnly,
    };
    let control = channel(
        vec![
            Ok(Some(preparation.clone())),
            Ok(Some(configure())),
            Ok(Some(preparation)),
            Ok(None),
        ],
        &trace,
        false,
    );
    let process = runner(
        &trace,
        vec![RunnerEvent::Exited(RunnerExit { code: Some(0) })],
    );
    let mut guest = supervisor(control, process, &trace);

    guest.run().await.unwrap();

    assert_eq!(
        *trace.lock().unwrap(),
        vec![
            hello(),
            configured(),
            Observation::Start,
            Observation::ControlLost,
            exited(Some(0)),
            Observation::Poweroff,
        ]
    );
    let warnings = String::from_utf8(warnings.0.lock().unwrap().clone()).unwrap();
    assert!(warnings.contains("runner configuration has not arrived"));
    assert!(warnings.contains("job preparation does not match the running job"));
    assert!(warnings.contains("job_id=inactive-job"));
}
