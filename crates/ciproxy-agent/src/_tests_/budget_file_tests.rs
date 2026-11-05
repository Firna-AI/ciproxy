//! Rejected writes cannot reach allocation. Accepted writes reserve rounded blocks first.

use std::sync::{Arc, Mutex};
use unimock::{MockFn, Unimock, matching};

use crate::budget_file::{BlockWriterMock, BudgetFile};
use crate::error::Error;
use crate::image_space::ImageSpaceMock;

#[test]
fn a_budget_refusal_stops_before_allocation_or_write() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let claims = events.clone();
    let allocations = events.clone();
    let writes = events.clone();
    let mock = Arc::new(Unimock::new((
        ImageSpaceMock::claim.stub(|each| {
            each.call(matching!(4096))
                .answers_arc(Arc::new(move |_, _| {
                    claims.lock().unwrap().push("reserve");
                    Ok(Arc::new(Unimock::new(())))
                }))
                .once()
                .then()
                .answers(&|_, _| Err(Error::ImageBudget));
        }),
        BlockWriterMock::allocate
            .each_call(matching!(0, 4096))
            .answers_arc(Arc::new(move |_, _, _| {
                allocations.lock().unwrap().push("allocate");
                Ok(())
            })),
        BlockWriterMock::write
            .some_call(matching!([1, 2, 3]))
            .answers_arc(Arc::new(move |_, _| {
                writes.lock().unwrap().push("write");
                Ok(())
            })),
    )));
    let mut output = BudgetFile::with_writer(Box::new((*mock).clone()), mock.clone());
    output.write(&[1, 2, 3]).unwrap();
    assert!(matches!(output.write(&[4]), Err(Error::ImageBudget)));
    assert_eq!(
        *events.lock().unwrap(),
        vec!["reserve", "allocate", "write"]
    );
}
