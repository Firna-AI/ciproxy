//! Root preparation keeps image payloads shared and changes only the VM copy.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::process::CommandsMock;
use crate::vm_root::{ReflinkRoot, RootDisk, RootFilesMock};

#[derive(Debug, Eq, PartialEq)]
enum Event {
    Command(String, Vec<String>),
    Extend(PathBuf, u64),
}

#[tokio::test]
async fn image_sized_total_reflinks_without_resize_or_a_second_disk() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let recorded = calls.clone();
    let mock = Arc::new(Unimock::new((
        RootFilesMock::length
            .some_call(matching!((path) if *path == image()))
            .returns(Ok(80_000_000_000)),
        CommandsMock::run
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, program, args, input| {
                assert!(input.is_empty());
                recorded
                    .lock()
                    .unwrap()
                    .push(Event::Command(program.to_owned(), args.to_vec()));
                Ok(Vec::new())
            }))
            .at_least_times(0),
    )));
    let root = ReflinkRoot {
        files: mock.clone(),
        commands: mock,
    };
    assert!(root.prepare(image(), copy(), 80_000_000_000).await.is_ok());
    assert_eq!(*calls.lock().unwrap(), vec![reflink()]);
}

#[tokio::test]
async fn larger_reserved_total_extends_and_grows_only_its_sparse_reflink_copy() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let commands = calls.clone();
    let files = calls.clone();
    let mock = Arc::new(Unimock::new((
        RootFilesMock::length
            .some_call(matching!((path) if *path == image()))
            .returns(Ok(80_000_000_000)),
        RootFilesMock::extend
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, path, bytes| {
                files
                    .lock()
                    .unwrap()
                    .push(Event::Extend(path.to_owned(), bytes));
                Ok(())
            }))
            .at_least_times(0),
        CommandsMock::run
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, program, args, input| {
                assert!(input.is_empty());
                commands
                    .lock()
                    .unwrap()
                    .push(Event::Command(program.to_owned(), args.to_vec()));
                Ok(Vec::new())
            })),
    )));
    let root = ReflinkRoot {
        files: mock.clone(),
        commands: mock,
    };
    assert!(root.prepare(image(), copy(), 100_000_000_000).await.is_ok());
    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            reflink(),
            Event::Extend(copy().to_owned(), 100_000_000_000),
            Event::Command(
                "resize2fs".into(),
                vec![copy().to_string_lossy().into_owned()]
            ),
            Event::Extend(copy().to_owned(), 100_000_000_000),
        ]
    );
}

#[tokio::test]
async fn total_below_image_refuses_without_copying_or_shrinking() {
    let mock = Arc::new(Unimock::new(
        RootFilesMock::length
            .some_call(matching!((path) if *path == image()))
            .returns(Ok(80_000_000_000)),
    ));
    let root = ReflinkRoot {
        files: mock.clone(),
        commands: mock,
    };
    assert!(matches!(
        root.prepare(image(), copy(), 79_999_999_999).await,
        Err(Error::Capacity)
    ));
}

fn image() -> &'static Path {
    Path::new("/images/verified/rootfs.ext4")
}

fn copy() -> &'static Path {
    Path::new("/jailer/vm/root/rootfs.ext4")
}

fn reflink() -> Event {
    Event::Command(
        "cp".into(),
        vec![
            "--reflink=always".into(),
            "--sparse=auto".into(),
            "--".into(),
            image().to_string_lossy().into_owned(),
            copy().to_string_lossy().into_owned(),
        ],
    )
}
