//! Mutable account facts and captured requests at the real-store dispatch boundary.

use std::sync::{Arc, Mutex};

use ciproxy_overflow::provider::{AccountFacts, BoatAccountApiMock, Capabilities};
use sandbox_interface::{
    BackendCreateSandboxRequest, SandboxAccountCapacityMock, SandboxLifecycleMock,
};
use unimock::{MockFn, Unimock, matching};

use super::fixtures::{capabilities, facts};

pub(super) struct Provider {
    pub(super) backend: Capabilities,
    pub(super) facts: Arc<Mutex<AccountFacts>>,
    pub(super) sizes: Arc<Mutex<Vec<String>>>,
    pub(super) creates: Arc<Mutex<Vec<BackendCreateSandboxRequest>>>,
    pub(super) recoveries: Arc<Mutex<Vec<BackendCreateSandboxRequest>>>,
    pub(super) reads: Arc<Mutex<(usize, usize)>>,
}

impl Provider {
    pub(super) fn new(trial: bool) -> Self {
        let facts = Arc::new(Mutex::new(AccountFacts { trial, ..facts() }));
        let sizes = Arc::new(Mutex::new(vec!["large".to_owned()]));
        let creates = Arc::new(Mutex::new(Vec::new()));
        let recoveries = Arc::new(Mutex::new(Vec::new()));
        let reads = Arc::new(Mutex::new((0, 0)));
        let account = facts.clone();
        let capacity = sizes.clone();
        let creating = creates.clone();
        let recovering = recoveries.clone();
        let account_reads = reads.clone();
        let capacity_reads = reads.clone();
        let backend = capabilities(
            Unimock::new((
                BoatAccountApiMock::account
                    .some_call(matching!())
                    .answers_arc(Arc::new(move |_| {
                        account_reads.lock().unwrap().0 += 1;
                        Ok(account.lock().unwrap().clone())
                    })),
                SandboxAccountCapacityMock::account_capacity
                    .some_call(matching!())
                    .answers_arc(Arc::new(move |_| {
                        capacity_reads.lock().unwrap().1 += 1;
                        Ok(capacity
                            .lock()
                            .unwrap()
                            .iter()
                            .map(|name| sandbox_boat::catalogue::offer(name).unwrap())
                            .collect())
                    })),
                SandboxLifecycleMock::create_sandbox
                    .some_call(matching!(_))
                    .answers_arc(Arc::new(move |_, request| {
                        creating.lock().unwrap().push(request);
                        Err(sandbox_interface::Error::BackendUnavailable {
                            backend_id: "boat".to_owned(),
                        })
                    })),
                SandboxLifecycleMock::recover_sandbox_create
                    .some_call(matching!(_))
                    .answers_arc(Arc::new(move |_, request| {
                        recovering.lock().unwrap().push(request);
                        Ok(None)
                    })),
            ))
            .no_verify_in_drop(),
        );
        Self {
            backend,
            facts,
            sizes,
            creates,
            recoveries,
            reads,
        }
    }
}
