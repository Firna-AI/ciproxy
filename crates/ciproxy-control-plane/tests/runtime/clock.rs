//! Injected deterministic runtime inputs.

use super::support::at;
use ciproxy_github::runtime::{Runtime, RuntimeMock};
use std::sync::Arc;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

pub(super) fn clock(with_ids: bool) -> Arc<dyn Runtime> {
    let now = RuntimeMock::now.each_call(matching!()).returns(at());
    if with_ids {
        Arc::new(Unimock::new((
            now,
            RuntimeMock::uuid
                .each_call(matching!())
                .answers(&|_| Uuid::from_u128(99)),
        )))
    } else {
        Arc::new(Unimock::new(now))
    }
}
