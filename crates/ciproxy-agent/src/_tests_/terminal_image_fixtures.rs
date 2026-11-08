//! Image storage writes fail once checked area cleanup removes their directory.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use unimock::{Clause, MockFn, matching};

use crate::error::ErrorResultExt;
use crate::image_fixtures::catalog;
use crate::image_store::ImageStoreMock;

pub(super) fn storage(
    area_present: Arc<AtomicBool>,
    events: Arc<Mutex<Vec<&'static str>>>,
) -> impl Clause {
    (
        ImageStoreMock::activate
            .some_call(matching!())
            .returns(Ok(())),
        ImageStoreMock::report
            .some_call(matching!())
            .returns(Ok(catalog())),
        ImageStoreMock::admit
            .some_call(matching!(0))
            .returns(Ok(())),
        ImageStoreMock::reclaim
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, _| {
                events.lock().unwrap().push("image-index");
                if area_present.load(Ordering::SeqCst) {
                    Ok(())
                } else {
                    Err(io::Error::from(io::ErrorKind::NotFound))
                        .context("write image index after area removal")
                }
            })),
    )
}
