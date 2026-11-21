//! Durable secret-free live probe state and explicit resource cleanup.

mod runner;
mod state_io;

use std::{path::Path, time::Duration};

use chrono::Utc;
use ciproxy_overflow::provider::Capabilities;
use sandbox_interface::{
    BackendCreateSandboxRequest, OperationId, ProviderRef, ResourceOwner, SandboxConsumer,
    SandboxId, SandboxLifetime, SandboxNetworkPolicy, SandboxSizeSelection,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Command;
use crate::live_support::{Result, SafeResult};

#[derive(Deserialize, Serialize)]
pub(super) struct State {
    delivery_id: Uuid,
    runner_id: Uuid,
    workspace_id: Uuid,
    snapshot: String,
    size: String,
    lifetime_seconds: u32,
    provider: Option<String>,
    phase: Phase,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
enum Phase {
    Creating,
    Preparing,
    Writing,
    Launching,
    Running,
    Deleting,
    Ended,
}

impl State {
    fn request(&self) -> BackendCreateSandboxRequest {
        BackendCreateSandboxRequest {
            sandbox_id: SandboxId::from_uuid(self.runner_id),
            operation_id: OperationId::from_uuid(self.delivery_id),
            owner: ResourceOwner::platform(self.workspace_id),
            consumer: SandboxConsumer::Runtime,
            lifetime: SandboxLifetime::OneShot {
                max_lifetime: Duration::from_secs(u64::from(self.lifetime_seconds)),
            },
            size: Some(SandboxSizeSelection::ProviderSize(self.size.clone())),
            deployment_id: format!("ciproxy-live/{}", self.workspace_id),
            profile: "ciproxy".to_owned(),
            network: SandboxNetworkPolicy::Open,
            snapshot_provider_ref: Some(ProviderRef::new(&self.snapshot)),
        }
    }

    fn provider(&self) -> Result<ProviderRef> {
        self.provider
            .as_ref()
            .map(ProviderRef::new)
            .ok_or_else(|| std::io::Error::other("provider pending"))
            .safe("resolve sandbox")
    }
}

pub(crate) async fn run(backend: Capabilities, command: Command) -> Result<()> {
    let path = match &command {
        Command::Account => None,
        Command::Create { state_file, .. }
        | Command::Recover { state_file }
        | Command::Runner { state_file, .. }
        | Command::Inspect { state_file }
        | Command::Usage { state_file }
        | Command::Delete { state_file } => Some(state_file),
    };
    let _lock = path.map(|path| state_io::lock(path)).transpose()?;
    match command {
        Command::Account => {
            let facts = backend.account.account().await.safe("check Boat account")?;
            let sizes = backend
                .capacity
                .account_capacity()
                .await
                .safe("read Boat sizes")?;
            println!(
                "Boat account: {}. Trial: {}. Eligible sizes: {}.",
                facts.name,
                facts.trial,
                sizes
                    .iter()
                    .map(|size| size.provider_size.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        Command::Create {
            state_file,
            snapshot,
            size,
            trial,
        } => {
            let existed = state_file.exists();
            let mut state = if existed {
                state_io::read(&state_file)?
            } else {
                State {
                    delivery_id: Uuid::new_v4(),
                    runner_id: Uuid::new_v4(),
                    workspace_id: Uuid::new_v4(),
                    snapshot,
                    size,
                    lifetime_seconds: if trial { 7200 } else { 21600 },
                    provider: None,
                    phase: Phase::Creating,
                }
            };
            state_io::save(&state_file, &state)?;
            if existed {
                recover(&backend, &state_file, &mut state).await?;
            } else {
                let created = backend
                    .lifecycle
                    .create_sandbox(state.request())
                    .await
                    .safe("dispatch Boat create")?;
                state.provider = Some(created.provider_ref.as_str().to_owned());
                state.phase = Phase::Preparing;
                state_io::save(&state_file, &state)?;
                println!("Sandbox identity saved. Use inspect, runner, usage, and delete.");
            }
        }
        Command::Recover { state_file } => {
            let mut state = state_io::read(&state_file)?;
            recover(&backend, &state_file, &mut state).await?;
        }
        Command::Runner {
            state_file,
            registration_file,
        } => runner::launch(&backend, &state_file, &registration_file).await?,
        Command::Inspect { state_file } => {
            let state: State = state_io::read(&state_file)?;
            let sandbox = backend
                .lifecycle
                .inspect_sandbox(state.provider()?)
                .await
                .safe("inspect Boat sandbox")?;
            println!("Sandbox state: {:?}.", sandbox.state);
        }
        Command::Usage { state_file } => {
            let state: State = state_io::read(&state_file)?;
            let usage = backend
                .account
                .usage(state.provider()?, None, Utc::now())
                .await
                .safe("read sandbox usage")?;
            if let Some(usage) = usage {
                println!("Runtime USD: {}. Running: {}.", usage.usd, usage.running);
            } else {
                println!("Boat no longer lists this sandbox's runtime.");
            }
        }
        Command::Delete { state_file } => {
            let mut state: State = state_io::read(&state_file)?;
            state.phase = Phase::Deleting;
            state_io::save(&state_file, &state)?;
            backend
                .lifecycle
                .destroy_sandbox(state.provider()?)
                .await
                .safe("delete Boat sandbox")?;
            state.phase = Phase::Ended;
            state_io::save(&state_file, &state)?;
            println!("Sandbox deletion confirmed.");
        }
    }
    Ok(())
}

async fn recover(backend: &Capabilities, path: &Path, state: &mut State) -> Result<()> {
    if state.provider.is_some() {
        println!("Sandbox identity is already saved.");
        return Ok(());
    }
    let recovered = backend
        .lifecycle
        .recover_sandbox_create(state.request())
        .await
        .safe("recover Boat create")?;
    if let Some(sandbox) = recovered {
        state.provider = Some(sandbox.provider_ref.as_str().to_owned());
        state.phase = Phase::Preparing;
        state_io::save(path, state)?;
        println!("Recovered sandbox identity. No create was repeated.");
    } else {
        println!("Create remains uncertain. Run recover again. No create was repeated.");
    }
    Ok(())
}
