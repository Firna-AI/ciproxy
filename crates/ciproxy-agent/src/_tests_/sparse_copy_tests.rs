//! In-memory extraction preserves zero bytes without writing zero-only blocks.

use std::io::{Cursor, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::sparse_copy::{ImageCopy, SparseCopy, SparseOutputMock};

#[test]
fn sparse_extraction_keeps_logical_bytes_and_length_without_writing_zero_runs() {
    let mut expected = vec![0; 4096];
    expected.extend_from_slice(&[7; 4096]);
    expected.extend_from_slice(&[0; 4096]);
    let output = Arc::new(Mutex::new(Cursor::new(Vec::new())));
    let data = output.clone();
    let holes = output.clone();
    let finished = output.clone();
    let written = Arc::new(Mutex::new(0));
    let observed = written.clone();
    let mut mock = Unimock::new((
        SparseOutputMock::data
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, bytes| {
                *observed.lock().unwrap() += bytes.len();
                data.lock().unwrap().write_all(bytes)
            })),
        SparseOutputMock::hole
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, bytes| {
                holes
                    .lock()
                    .unwrap()
                    .seek(SeekFrom::Current(i64::from(bytes)))?;
                Ok(())
            })),
        SparseOutputMock::finish
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, length| {
                finished
                    .lock()
                    .unwrap()
                    .get_mut()
                    .resize(length as usize, 0);
                Ok(())
            })),
    ));
    let bytes = SparseCopy
        .copy(&mut Cursor::new(expected.clone()), &mut mock)
        .unwrap();
    assert_eq!(bytes, 12_288);
    assert_eq!(*output.lock().unwrap().get_ref(), expected);
    assert_eq!(*written.lock().unwrap(), 4096);
}
