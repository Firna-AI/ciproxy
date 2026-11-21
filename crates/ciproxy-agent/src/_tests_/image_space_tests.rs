//! Every writer shares physical observations and reservations before allocation.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::image_space::{ImageBudget, ImageSpace, SpaceUsageMock};

#[test]
fn pending_reservations_and_recovered_allocation_cannot_borrow_job_space() {
    let used = Arc::new(Mutex::new(200));
    let observed = used.clone();
    let mock = Arc::new(Unimock::new(
        SpaceUsageMock::used
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| Ok(*observed.lock().unwrap()))),
    ));
    let budget = ImageBudget::new(500, mock.clone());
    let first = budget.claim(200).unwrap();
    assert!(matches!(budget.claim(101), Err(Error::ImageBudget)));
    let second = budget.claim(100).unwrap();
    assert!(matches!(budget.claim(1), Err(Error::ImageBudget)));
    let release = || {
        let _end = (first, second);
    };
    release();
    *used.lock().unwrap() = 499;
    assert!(budget.claim(1).is_ok());
    assert!(matches!(budget.claim(2), Err(Error::ImageBudget)));
    *used.lock().unwrap() = 501;
    assert!(matches!(budget.claim(0), Err(Error::ImageBudget)));
}
