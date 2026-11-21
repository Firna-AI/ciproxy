//! Eviction precedes admission. A refused update keeps the usable current base.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::boundaries::Images;
use crate::error::Error;
use crate::image_fixtures::{catalog, image};
use crate::image_store::ImageStoreMock;
use crate::images::ManagedImages;

#[tokio::test]
async fn fetch_reclaims_unused_entries_before_admission_and_keeps_current_on_refusal() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let reclaim = events.clone();
    let admit = events.clone();
    let mock = Arc::new(Unimock::new((
        ImageStoreMock::activate.some_call(matching!()).returns(Ok(())),
        ImageStoreMock::report.some_call(matching!()).returns(Ok(catalog())),
        ImageStoreMock::reclaim.each_call(matching!((keep, pending) if keep.len() == 1 && keep[0].0 == "old" && pending.is_empty()))
            .answers_arc(Arc::new(move |_, _, _| { reclaim.lock().unwrap().push("reclaim"); Ok(()) })),
        ImageStoreMock::admit.each_call(matching!(_)).answers_arc(Arc::new(move |_, bytes| {
            admit.lock().unwrap().push("admit"); if bytes == 0 { Ok(()) } else { Err(Error::ImageBudget) }
        })),
    )));
    let images = ManagedImages::new(mock.clone(), mock.clone());
    images.recover().unwrap();
    let observed = events.lock().unwrap().clone();
    assert_eq!(observed, vec!["reclaim", "admit"]);
    let held = images.hold_newest().unwrap();
    let result = images.fetch(&image()).await;
    assert!(matches!(result, Err(Error::ImageBudget)));
    let observed = events.lock().unwrap().clone();
    assert_eq!(observed, vec!["reclaim", "admit", "reclaim", "admit"]);
    assert_eq!(images.report().unwrap(), catalog());
    assert_eq!(held.metadata().image_id.0, "old");
}
