//! Restartable scale-set listeners and durable lifecycle replay.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use async_trait::async_trait;
use ciproxy_github::listener::{DurableListener, JobOutput, ListenerOptions, ScaleSetListener};
use ciproxy_github::persistence::session::ListenerTarget;
use ciproxy_store_interface::github::GithubJob;
use ciproxy_store_interface::organizations::AppState;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::{MissedTickBehavior, interval};

use crate::accounting::Accounting;
use crate::composition::Hosted;
use crate::error::{ErrorResultExt, Result};
use crate::runtime::OwnMachines;
use crate::tasks::OwnedTask;

struct DiscoveredTarget {
    target: ListenerTarget,
    installation_id: i64,
}

struct Running {
    stop: watch::Sender<bool>,
    task: OwnedTask<()>,
}

impl Drop for Running {
    fn drop(&mut self) {
        self.stop.send_replace(true);
    }
}

struct Readiness(Arc<std::sync::atomic::AtomicBool>);
impl Drop for Readiness {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

struct RecoveryTask(JoinHandle<()>);
impl Drop for RecoveryTask {
    fn drop(&mut self) {
        self.0.abort();
    }
}

struct Jobs {
    machines: Arc<dyn OwnMachines>,
    accounting: Arc<dyn Accounting>,
    workspace_id: uuid::Uuid,
}

#[async_trait]
impl JobOutput for Jobs {
    async fn changed(&self, job: GithubJob) -> ciproxy_github::error::Result<()> {
        let result = self.machines.job(self.workspace_id, job).await;
        self.accounting.request();
        match result {
            Ok(()) => Ok(()),
            Err(_) => Err(ciproxy_github::error::Error::OrganizationInactive),
        }
    }
}

impl Hosted {
    /// Run periodic reconciliation until shutdown, closing message sessions first.
    pub async fn supervise(&self, mut stop: watch::Receiver<bool>) -> Result<()> {
        let _readiness = Readiness(self.ready.clone());
        let recovery = self.recovery.clone();
        let mut recovery_stop = stop.clone();
        let _recovery = RecoveryTask(tokio::spawn(async move {
            let mut ticks = interval(Duration::from_secs(60));
            loop {
                tokio::select! {
                    _ = ticks.tick() => {
                        tokio::select! {
                            result = recovery.retry_failed() => if let Err(error) = result { tracing::warn!(%error,"GitHub webhook history recovery will retry"); },
                            _ = recovery_stop.changed() => return,
                        }
                    },
                    _ = recovery_stop.changed() => return,
                }
            }
        }));
        let repositories = self.repositories.clone();
        let mut metadata_stop = stop.clone();
        let _metadata = RecoveryTask(tokio::spawn(async move {
            let mut ticks = interval(Duration::from_secs(300));
            loop {
                tokio::select! {
                    _=ticks.tick()=>{tokio::select!{
                        result=repositories.refresh()=>if let Err(error)=result{tracing::warn!(%error,"repository snapshot refresh will retry");},
                        _=metadata_stop.changed()=>return,
                    }},
                    _=metadata_stop.changed()=>return,
                }
            }
        }));
        let worker = self.accounting.clone();
        let receiver = stop.clone();
        let mut accounting = OwnedTask(tokio::spawn(async move { worker.run(receiver).await }));
        let mut running: HashMap<uuid::Uuid, Running> = HashMap::new();
        let mut ticks = interval(Duration::from_secs(10));
        ticks.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = ticks.tick() => {
                    if accounting.0.is_finished() {
                        let worker = self.accounting.clone();
                        let receiver = stop.clone();
                        accounting = OwnedTask(tokio::spawn(async move { worker.run(receiver).await }));
                    }
                    self.accounting.request();
                    self.machines.tick().await?;
                    for github_id in self.app.pending_lifecycle_organizations().await.internal()? {
                        if let Err(error) = self.app.replay_lifecycle(github_id).await { tracing::warn!(%github_id, %error, "lifecycle replay will retry"); }
                    }
                    let targets = self.listener_targets().await?;
                    {
                        let stopping = stop.borrow();
                        if !*stopping {
                            self.ready.store(true, Ordering::Release);
                        }
                    }
                    let stale: Vec<_> = running.iter().filter(|(id, value)| value.task.0.is_finished() || !targets.iter().any(|discovered| discovered.target.scale_set_id == **id)).map(|(id, _)| *id).collect();
                    for id in stale {
                        if let Some(mut value) = running.remove(&id) { value.stop.send_replace(true); let _ = (&mut value.task.0).await; }
                    }
                    for DiscoveredTarget { target, installation_id } in targets {
                        if running.contains_key(&target.scale_set_id) { continue; }
                        let listener: Arc<dyn ScaleSetListener> = Arc::new(DurableListener { store: self.persistence.clone(), rest: self.rest.clone(), scale_sets: self.scale_sets.clone(), output: Arc::new(Jobs { machines: self.machines.clone(), accounting: self.accounting.clone(), workspace_id: target.workspace_id }) });
                        let app = self.app.clone();
                        let (cancel, receiver) = watch::channel(false);
                        let task = tokio::spawn(async move {
                            let result = listener.run(target, ListenerOptions { owner: format!("ciproxy-{}", std::process::id()), capacity: 10_000, max_polls: None }, receiver).await;
                            if let Err(error) = result {
                                if matches!(error, ciproxy_github::error::Error::ScaleSetMissing { .. }) { let _ = app.refresh_repositories(installation_id).await; }
                                tracing::warn!(%error, "scale-set listener will restart");
                            }
                        });
                        running.insert(target.scale_set_id, Running { stop: cancel, task:OwnedTask(task) });
                    }
                }
                changed = stop.changed() => { if changed.is_err() || *stop.borrow() { break; } }
            }
        }
        for value in running.values() {
            value.stop.send_replace(true);
        }
        for (_, mut value) in running {
            let _ = (&mut value.task.0).await;
        }
        Ok(())
    }

    async fn listener_targets(&self) -> Result<Vec<DiscoveredTarget>> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || {
            let mut targets = Vec::new();
            for workspace_id in store.routing_workspaces().internal()? {
                for organization in store.organizations(workspace_id).internal()? {
                    if organization.app_state != AppState::Active {
                        continue;
                    }
                    for binding in store
                        .listener_scale_sets(workspace_id, organization.id)
                        .internal()?
                    {
                        targets.push(DiscoveredTarget {
                            target: ListenerTarget {
                                workspace_id,
                                organization_id: organization.id,
                                scale_set_id: binding.scale_set.id,
                            },
                            installation_id: organization.installation_id,
                        });
                    }
                }
            }
            Ok(targets)
        })
        .await
        .internal()?
    }
}
