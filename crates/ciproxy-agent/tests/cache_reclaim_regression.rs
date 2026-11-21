//! Real filesystem proof that interrupted cache migration cannot retain archives.

#![expect(
    dead_code,
    reason = "The harness imports only the cache-file entry point."
)]

use std::fs;
use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use cache_admin::{CacheFiles, OwnedCacheFiles};
use process::CommandsMock;

#[path = "../src/cache_admin.rs"]
mod cache_admin;
#[path = "../src/error.rs"]
mod error;
#[path = "../src/process.rs"]
mod process;

#[tokio::test]
async fn interrupted_cache_data_migration_reclaims_both_layouts_before_acknowledgement() {
    let root = std::env::temp_dir().join(format!("ciproxy-cache-{}", uuid::Uuid::new_v4()));
    let child = root.join("data");
    fs::create_dir_all(&child).unwrap();
    let legacy = root.join("legacy.blob");
    let migrated = child.join("migrated.blob");
    fs::write(&legacy, b"public legacy archive").unwrap();
    fs::write(&migrated, b"public migrated archive").unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let captured = calls.clone();
    let cache = OwnedCacheFiles {
        directory: root.clone(),
        commands: Arc::new(Unimock::new(
            CommandsMock::run
                .each_call(matching!(_, _, _))
                .answers_arc(Arc::new(move |_, program, args, _| {
                    assert_eq!(program, "/usr/local/bin/ciproxy-cache-sidecar");
                    assert_eq!(&args[..2], &["reclaim", "--cache-dir"]);
                    let directory = std::path::Path::new(&args[2]);
                    captured.lock().unwrap().push(directory.to_path_buf());
                    for file in fs::read_dir(directory).unwrap() {
                        let path = file.unwrap().path();
                        if path
                            .extension()
                            .is_some_and(|extension| extension == "blob")
                        {
                            fs::remove_file(path).unwrap();
                        }
                    }
                    Ok(Vec::new())
                })),
        )),
    };
    let result = cache.release_occupied_space().await;
    let remains = [legacy.exists(), migrated.exists()];
    fs::remove_dir_all(&root).unwrap();
    result.unwrap();
    assert_eq!(
        remains,
        [false, false],
        "cache recovery acknowledged an unreleased archive"
    );
    assert_eq!(*calls.lock().unwrap(), vec![root, child]);
}
