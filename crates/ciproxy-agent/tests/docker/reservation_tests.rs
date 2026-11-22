//! Admission evicts idle bases and lowers cache space before each disk write.

use std::fs::{self, File};
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

use unimock::{MockFn, Unimock, matching};

use crate::boundaries::CacheMock;
use crate::boundaries::DockerDisks;
use crate::docker_files;
use crate::error::Error;
use crate::fixtures::{TemporaryLayout, facts, scope, vm};
use crate::process::CommandsMock;

#[tokio::test]
async fn docker_creation_and_each_growth_reclaim_oldest_idle_bases_before_writes() {
    let temporary = TemporaryLayout::new();
    let record = vm("reclaim-growth");
    fs::create_dir_all(temporary.layout.jail_root(&record)).unwrap();
    let mut bases = Vec::new();
    for (name, bytes, order) in [
        ("oldest", 8192, 1),
        ("middle", 24576, 2),
        ("newest", 32768, 3),
    ] {
        let base = temporary.base(name);
        fs::create_dir_all(base.parent().unwrap()).unwrap();
        fs::write(&base, vec![b'B'; bytes]).unwrap();
        File::open(&base)
            .unwrap()
            .set_modified(UNIX_EPOCH + Duration::from_secs(order))
            .unwrap();
        bases.push(base);
    }
    let idle = bases.clone();
    let budgets = Arc::new(Mutex::new(Vec::new()));
    let captured = budgets.clone();
    let latest = budgets.clone();
    let writes = Arc::new(Mutex::new(Vec::new()));
    let observed = writes.clone();
    let layout = temporary.layout.clone();
    let samples = Arc::new(Mutex::new(0));
    let allocation = Arc::new(tokio::sync::Mutex::new(()));
    let budgets_lock = allocation.clone();
    let writes_lock = allocation.clone();
    let mut adapter = temporary.adapter(Unimock::new((
        CacheMock::budget
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, bytes| {
                assert!(
                    budgets_lock.try_lock().is_err(),
                    "lease and cache release must hold the admission lock"
                );
                captured.lock().unwrap().push(bytes);
                Ok(())
            })),
        CommandsMock::run
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, program, _, _| {
                assert!(
                    writes_lock.try_lock().is_ok(),
                    "filesystem commands must permit first reports"
                );
                if program == "dumpe2fs" {
                    let mut sample = samples.lock().unwrap();
                    *sample += 1;
                    return if *sample < 3 {
                        Ok(b"Free blocks: 0\nBlock size: 4096\n".to_vec())
                    } else {
                        Err(Error::HostCommand {
                            program: "measurement-stop".to_owned(),
                        })
                    };
                }
                let held = docker_files::held(&layout).unwrap();
                let exists = idle.iter().map(|path| path.exists()).collect::<Vec<_>>();
                let budget = *latest.lock().unwrap().last().unwrap();
                observed
                    .lock()
                    .unwrap()
                    .push((program.to_owned(), held, exists, budget));
                Ok(Vec::new())
            })),
    )));
    adapter.allocation = allocation;
    let result = adapter
        .prepare(&record, &facts(), &scope(), 4096, 65536)
        .await;
    assert!(matches!(result, Err(Error::HostCommand { program }) if program == "measurement-stop"));
    assert_eq!(
        *writes.lock().unwrap(),
        vec![
            ("mkfs.ext4".to_owned(), 8192, vec![false, true, true], 0),
            (
                "resize2fs".to_owned(),
                24576,
                vec![false, false, true],
                8192
            ),
            (
                "resize2fs".to_owned(),
                40960,
                vec![false, false, false],
                24576
            ),
        ]
    );
    assert_eq!(adapter.report().unwrap().leased_bytes, 0);
    assert_eq!(
        bases.iter().map(|path| path.exists()).collect::<Vec<_>>(),
        vec![false, false, false]
    );
}
