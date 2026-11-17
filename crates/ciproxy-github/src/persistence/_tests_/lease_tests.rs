//! Dedicated lease checks must leave the asynchronous runtime worker.

use std::sync::{Arc, Mutex};
use std::thread;

use unimock::{MockFn, Unimock, matching};

use super::{ConnectionMock, Handle, Lease};

#[tokio::test(flavor = "current_thread")]
async fn lease_check_runs_outside_the_runtime_thread() {
    let runtime_thread = thread::current().id();
    let checked_on = Arc::new(Mutex::new(None));
    let observed = checked_on.clone();
    let connection = Unimock::new(
        ConnectionMock::check_connection
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                *observed.lock().unwrap() = Some(thread::current().id());
                Ok(())
            })),
    );
    let lease: Box<dyn Lease> = Box::new(Handle::new(Box::new(connection)));

    lease.check().await.unwrap();

    assert_ne!(checked_on.lock().unwrap().unwrap(), runtime_thread);
}
