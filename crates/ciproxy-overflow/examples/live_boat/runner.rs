//! Fixed privileged guest smoke with a private registration file and launch fence.

use std::{fs, path::Path};

use ciproxy_overflow::provider::{Capabilities, registration_root};
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::secret::Secret;
use sandbox_interface::BackendWriteFileRequest;
use serde::{Deserialize, Serialize};

use crate::live_support::{Result, SafeResult};

use super::{Phase, State, state_io};

#[derive(Deserialize, Serialize)]
struct Registration {
    configure: AgentToGuest,
    exit_report_url: Secret,
    exit_report_token: Secret,
}

pub(super) async fn launch(
    backend: &Capabilities,
    state_path: &Path,
    registration_path: &Path,
) -> Result<()> {
    let mut state: State = state_io::read(state_path)?;
    let sandbox = state.provider()?;
    if state.phase == Phase::Writing {
        state.phase = Phase::Deleting;
        state_io::save(state_path, &state)?;
        backend
            .lifecycle
            .destroy_sandbox(sandbox)
            .await
            .safe("delete interrupted write")?;
        state.phase = Phase::Ended;
        state_io::save(state_path, &state)?;
        println!("Interrupted file transfer ended. Sandbox deletion confirmed.");
        return Ok(());
    }
    if state.phase == Phase::Running {
        println!("Guest launch was already accepted.");
        return Ok(());
    }
    if state.phase == Phase::Preparing {
        let mut registration: Registration =
            serde_json::from_slice(&fs::read(registration_path).safe("read private registration")?)
                .safe("decode private registration")?;
        if let AgentToGuest::ConfigureRunner {
            runner_id, cache, ..
        } = &mut registration.configure
        {
            runner_id.0 = state.runner_id.to_string();
            *cache = None;
        } else {
            return Err(std::io::Error::other("configure required"))
                .safe("check private registration");
        }
        state.phase = Phase::Writing;
        state_io::save(state_path, &state)?;
        backend
            .files
            .write_file(BackendWriteFileRequest {
                sandbox_provider_ref: sandbox.clone(),
                root: "/workspace".to_owned(),
                path: format!(".ciproxy-bootstrap-{}", state.delivery_id.simple()),
                bytes: Vec::new(),
            })
            .await
            .safe("bootstrap sandbox runtime")?;
        backend
            .launcher
            .prepare(sandbox.clone(), state.delivery_id)
            .await
            .safe("prepare private input")?;
        let result = backend
            .files
            .write_file(BackendWriteFileRequest {
                sandbox_provider_ref: sandbox.clone(),
                root: registration_root(state.delivery_id),
                path: "registration.json".to_owned(),
                bytes: serde_json::to_vec(&registration).safe("encode private input")?,
            })
            .await;
        if matches!(result, Err(sandbox_interface::Error::FileWriteUnconfirmed)) {
            state.phase = Phase::Deleting;
            state_io::save(state_path, &state)?;
            backend
                .lifecycle
                .destroy_sandbox(sandbox)
                .await
                .safe("delete uncertain write")?;
            state.phase = Phase::Ended;
            state_io::save(state_path, &state)?;
            return Err(std::io::Error::other("write unconfirmed")).safe("stage registration");
        }
        result.safe("stage registration")?;
        state.phase = Phase::Launching;
        state_io::save(state_path, &state)?;
    }
    if state.phase == Phase::Launching {
        backend
            .launcher
            .launch(sandbox, state.delivery_id)
            .await
            .safe("launch guest")?;
        state.phase = Phase::Running;
        state_io::save(state_path, &state)?;
        println!("Guest launch accepted. Check the GitHub job, then run usage and delete.");
    }
    Ok(())
}
