//! Session ownership, restart delivery, cancellation, and ACK ordering.

use ciproxy_store_interface::organizations::AppState;
use scaleset::MessageId;
use tokio::sync::watch;

use crate::error::{Error, Result};
use crate::persistence::session::{ListenerContext, ListenerTarget, SessionLease};
use crate::scale_sets::MessageQueue;

use super::{DurableListener, ListenerOptions};

impl DurableListener {
    pub(super) async fn listen(
        &self,
        target: ListenerTarget,
        options: ListenerOptions,
        stop: watch::Receiver<bool>,
    ) -> Result<()> {
        let Some(lease) = self.store.lock(target).await? else {
            return Err(Error::SessionOwned);
        };
        let context = self.check(target, &*lease).await?;
        let api = self
            .scale_sets
            .for_organization(&context.organization)
            .await?;
        let queue = api
            .open(context.scale_set.github_id, &options.owner)
            .await?;
        let result = self
            .poll(
                target,
                &options,
                stop,
                &*lease,
                &*queue,
                context.scale_set.github_id,
            )
            .await;
        let closed = queue.close().await;
        result?;
        closed
    }

    async fn check(
        &self,
        target: ListenerTarget,
        lease: &dyn SessionLease,
    ) -> Result<ListenerContext> {
        lease.check().await?;
        let context = self.store.context(target).await?;
        if context.organization.app_state != AppState::Active {
            return Err(Error::OrganizationInactive);
        }
        Ok(context)
    }

    async fn bound_context(
        &self,
        target: ListenerTarget,
        lease: &dyn SessionLease,
        opened: i64,
    ) -> Result<ListenerContext> {
        let context = self.check(target, lease).await?;
        let current = context.scale_set.github_id;
        if current != opened {
            return Err(Error::ScaleSetRestart { opened, current });
        }
        Ok(context)
    }

    async fn poll(
        &self,
        target: ListenerTarget,
        options: &ListenerOptions,
        mut stop: watch::Receiver<bool>,
        lease: &dyn SessionLease,
        queue: &dyn MessageQueue,
        opened: i64,
    ) -> Result<()> {
        let mut last_message: Option<MessageId> = None;
        let mut polls = 0;
        loop {
            if *stop.borrow() || options.max_polls.is_some_and(|limit| polls >= limit) {
                return Ok(());
            }
            let context = self.bound_context(target, lease, opened).await?;
            self.deliver(target).await?;
            let polled = tokio::select! {
                _ = stop.changed() => return Ok(()),
                result = queue.poll(last_message, options.capacity) => result,
            };
            self.bound_context(target, lease, opened).await?;
            let message = polled?;
            polls += 1;
            let Some(message) = message else {
                continue;
            };
            self.persist(&context, &message).await?;
            let requests: Vec<_> = message
                .jobs_available()
                .map(|job| job.job.runner_request_id)
                .filter(|id| id.get() > 0)
                .collect();
            if !requests.is_empty() {
                queue.acquire(&requests).await?;
            }
            self.deliver(target).await?;
            self.bound_context(target, lease, opened).await?;
            queue.acknowledge(message.id).await?;
            last_message = Some(message.id);
        }
    }

    async fn deliver(&self, target: ListenerTarget) -> Result<()> {
        for delivery in self.store.pending(target).await? {
            let job_id = delivery.job.id;
            self.output.changed(delivery.job).await?;
            self.store
                .confirm(target, job_id, delivery.revision)
                .await?;
        }
        Ok(())
    }
}
