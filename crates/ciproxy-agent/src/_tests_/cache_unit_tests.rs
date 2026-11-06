//! Every managed listener start applies the policy installed with its unit.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::cache_network::RULES;
use crate::cache_unit::{
    CacheUnit, CacheUnitFilesMock, InstalledCacheUnit, POLICY_PATH, UNIT_PATH,
};

#[test]
fn installed_cache_unit_requires_the_written_policy_before_its_listener() {
    let writes = Arc::new(Mutex::new(Vec::new()));
    let written = writes.clone();
    let unit = InstalledCacheUnit {
        files: Arc::new(Unimock::new(
            CacheUnitFilesMock::write
                .each_call(matching!(_, _))
                .answers_arc(Arc::new(move |_, path, contents| {
                    written
                        .lock()
                        .unwrap()
                        .push((path.to_owned(), contents.to_vec()));
                    Ok(())
                })),
        )),
    };
    unit.install().unwrap();
    let installed = writes.lock().unwrap();
    let unit = installed
        .iter()
        .find(|(path, _)| path == UNIT_PATH)
        .unwrap();
    let lines: Vec<_> = std::str::from_utf8(&unit.1).unwrap().lines().collect();
    let expected = format!("ExecStartPre=/usr/sbin/nft -f {POLICY_PATH}");
    assert!(lines.contains(&expected.as_str()));
    let prepare = lines.iter().position(|line| *line == expected).unwrap();
    let listener = lines
        .iter()
        .position(|line| line.starts_with("ExecStart="))
        .unwrap();
    assert!(prepare < listener);
    assert_eq!(
        installed.first(),
        Some(&(POLICY_PATH.to_owned(), RULES.as_bytes().to_vec()))
    );
}
