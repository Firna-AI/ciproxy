//! No machine cache on macOS yet: every job uses the GitHub cache.
//!
//! `prepare` gives no cache session, so the core sends the GitHub-only
//! decision to the guest. The steps that end or clean a session have
//! nothing to do. The report says that the cache is unavailable.

use async_trait::async_trait;
use ciproxy_protocol::cache::{CacheConnection, MachineCacheReport};
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::jobs::{JobFacts, JobScope};

use crate::boundaries::Cache;
use crate::cache_authority::CheckedRuntime;
use crate::error::{Error, Result};
use crate::state::VmRecord;

pub(crate) struct NoMachineCache;

#[async_trait]
impl Cache for NoMachineCache {
    async fn refresh_budget(&self, _capacity_bytes: u64) -> Result<()> {
        Ok(())
    }

    async fn fallback(&self, _vm: &VmRecord) -> Result<()> {
        Ok(())
    }

    async fn budget(&self, _capacity_bytes: u64) -> Result<()> {
        Ok(())
    }

    async fn report(&self) -> Result<MachineCacheReport> {
        Err(Error::CacheUnavailable)
    }

    async fn clear(&self) -> Result<()> {
        Err(Error::CacheUnavailable)
    }

    async fn release(&self, _vm: &VmRecord) -> Result<()> {
        Ok(())
    }

    async fn handoff(&self, _vm: &VmRecord, _job: &JobId, _runtime: CheckedRuntime) -> Result<()> {
        Ok(())
    }

    async fn prepare(
        &self,
        _runner: &RunnerId,
        _facts: &JobFacts,
    ) -> Result<Option<(CacheConnection, JobScope, u64)>> {
        Ok(None)
    }
}
