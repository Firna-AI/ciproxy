//! Replay host-authorized preparation to each reconnected source-hook socket.

use async_trait::async_trait;
use ciproxy_protocol::guest::{JobDecision, JobPreparation};

use crate::assignment::{Assignment, WorkerReply};
use crate::error::Result;
use crate::handoff::{Handoff, PreparedProxy, prepare};
use crate::preparation::{Prepared, PreparedJob};

#[async_trait]
pub(crate) trait AssignedJob: Send {
    fn handoff(&self) -> &Handoff;
    fn reconnect(&mut self, assignment: Assignment) -> bool;
    async fn decide(&mut self, decision: JobDecision) -> Result<JobPreparation>;
}

pub(crate) struct WaitingJob {
    handoff: Handoff,
    reply: Option<Box<dyn WorkerReply>>,
    prepared: Option<Box<dyn PreparedJob>>,
    github_only: bool,
}

impl WaitingJob {
    pub(crate) fn new(assignment: Assignment) -> Self {
        Self {
            handoff: assignment.handoff,
            reply: Some(assignment.reply),
            prepared: None,
            github_only: false,
        }
    }
}

#[async_trait]
impl AssignedJob for WaitingJob {
    fn handoff(&self) -> &Handoff {
        &self.handoff
    }

    fn reconnect(&mut self, assignment: Assignment) -> bool {
        if assignment.handoff.job_id != self.handoff.job_id {
            return false;
        }
        self.reply = Some(assignment.reply);
        true
    }

    async fn decide(&mut self, decision: JobDecision) -> Result<JobPreparation> {
        if decision == JobDecision::GitHubOnly {
            self.github_only = true;
        }
        if self.github_only {
            let mut outcome = match &mut self.prepared {
                Some(prepared) => prepared.replay(None).await,
                None => JobPreparation::default(),
            };
            outcome.cache_ready = false;
            if let Some(reply) = &mut self.reply
                && reply.ready(None).await.is_err()
            {
                tracing::warn!("GitHub-only decision will replay after socket reconnect");
            }
            return Ok(outcome);
        }
        let JobDecision::MachineCache {
            facts,
            cache,
            docker,
        } = decision
        else {
            return Ok(JobPreparation::default());
        };
        if let Some(prepared) = &mut self.prepared {
            let outcome = prepared.replay(Some(cache)).await;
            if outcome.cache_ready
                && let Some(reply) = &mut self.reply
                && reply
                    .ready(Some(&self.handoff.runtime.proxy_url))
                    .await
                    .is_err()
            {
                tracing::warn!("cache-ready decision will replay after socket reconnect");
            }
            return Ok(outcome);
        }
        let connection = Some(cache.clone());
        match prepare(
            self.handoff.clone(),
            self.reply.take(),
            *facts,
            cache,
            docker,
        )
        .await
        {
            Ok(initial) => {
                let outcome = initial.outcome;
                self.prepared = Some(Box::new(Prepared {
                    initial,
                    connection,
                }));
                Ok(outcome)
            }
            Err(error) => {
                self.prepared = Some(Box::new(Prepared {
                    initial: PreparedProxy {
                        proxy: None,
                        outcome: JobPreparation::default(),
                    },
                    connection: None,
                }));
                Err(error)
            }
        }
    }
}

#[cfg(test)]
#[path = "_tests_/runner_assignment_tests.rs"]
mod runner_assignment_tests;
