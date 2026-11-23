//! Failed section reads preserve the rest of a machine's current state.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::cache::DockerDiskReport;
use ciproxy_protocol::images::ImageReport;
use serde_json::json;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::DockerDisksMock;
use crate::boundaries::{CacheMock, HostMock, ImagesMock};
use crate::controller::Agent;
use crate::error::Error;
use crate::fixtures::{controller, empty_cache, hardware, vm};
use crate::state::MachineState;

#[tokio::test]
async fn failed_statistics_reads_keep_machine_and_runner_reports_available() {
    for (cache_failed, docker_failed) in
        [(true, true), (true, false), (false, true), (false, false)]
    {
        let mock = Unimock::new((
            HostMock::hardware
                .some_call(matching!())
                .returns(Ok(hardware())),
            HostMock::hostname
                .some_call(matching!())
                .returns(Ok("measured-host".into())),
            ImagesMock::report
                .some_call(matching!())
                .returns(Ok(ImageReport {
                    available: vec![],
                    newest: None,
                })),
            CacheMock::report
                .some_call(matching!())
                .returns(if cache_failed {
                    Err(Error::CacheUnavailable)
                } else {
                    Ok(empty_cache())
                }),
            DockerDisksMock::report
                .some_call(matching!())
                .returns(if docker_failed {
                    Err(Error::Capacity)
                } else {
                    Ok(DockerDiskReport {
                        leased_bytes: 123,
                        repositories: 2,
                        used_bytes: 900,
                    })
                }),
        ));
        let mut agent = controller(
            MachineState {
                runners: vec![vm("running", true)],
                ..MachineState::default()
            },
            mock.clone(),
        );
        agent.dockers = Some(Arc::new(mock));
        let report = agent.report().await.unwrap();
        let wire = serde_json::to_value(&report).unwrap();
        let expected_cache = if cache_failed {
            json!({"status": "unavailable"})
        } else {
            json!({"status":"measured", "used_bytes":0, "capacity_bytes":0,
                "restores_last_7_days":0, "hits_last_7_days":0, "unknown_jobs":[], "jobs":[]})
        };
        let expected_docker = if docker_failed {
            json!({"status": "unavailable"})
        } else {
            json!({"status":"measured", "leased_bytes":123, "repositories":2, "used_bytes":900})
        };
        assert_eq!(
            wire["cache"], expected_cache,
            "cache failed: {cache_failed}, Docker failed: {docker_failed}"
        );
        assert_eq!(
            wire["docker_disks"], expected_docker,
            "cache failed: {cache_failed}, Docker failed: {docker_failed}"
        );
        assert_eq!(report.hostname, "measured-host");
        assert_eq!(report.runners, vec![vm("running", true).report]);
    }
}

#[tokio::test]
async fn first_public_report_sends_budget_without_reclaiming_docker_disks() {
    for budget_available in [true, false] {
        let events = Arc::new(Mutex::new(Vec::new()));
        let budgets = events.clone();
        let reports = events.clone();
        let mock = Unimock::new((
            HostMock::hardware
                .each_call(matching!())
                .answers(&|_| Ok(hardware())),
            HostMock::hostname
                .each_call(matching!())
                .answers(&|_| Ok("first-report".into())),
            ImagesMock::report.each_call(matching!()).answers(&|_| {
                Ok(ImageReport {
                    available: vec![],
                    newest: None,
                })
            }),
            DockerDisksMock::report
                .each_call(matching!())
                .answers(&|_| {
                    Ok(DockerDiskReport {
                        leased_bytes: 100_000_000_000,
                        repositories: 1,
                        used_bytes: 1,
                    })
                }),
            DockerDisksMock::unused_bytes
                .each_call(matching!())
                .answers(&|_| Ok(20_000_000_000))
                .at_least_times(0),
            CacheMock::refresh_budget
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, bytes| {
                    budgets.lock().unwrap().push(format!("budget:{bytes}"));
                    if budget_available {
                        Ok(())
                    } else {
                        Err(Error::CacheUnavailable)
                    }
                }))
                .at_least_times(0),
            CacheMock::report
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    reports.lock().unwrap().push("report".into());
                    Ok(empty_cache())
                })),
        ));
        let mut agent = controller(
            MachineState {
                runners: vec![vm("running", true)],
                ..MachineState::default()
            },
            mock.clone(),
        );
        agent.dockers = Some(Arc::new(mock));
        agent.reporting_initialized = tokio::sync::Mutex::new(false);
        agent.report().await.unwrap();
        agent.report().await.unwrap();
        assert_eq!(
            *events.lock().unwrap(),
            vec!["budget:444100387328", "report", "report"]
        );
    }
}
