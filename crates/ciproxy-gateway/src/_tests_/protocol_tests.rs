//! Version and platform rejections before a persistent side effect.

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_store_interface::error::Error as StoreError;
use ciproxy_store_interface::gateway::GatewayStoreMock;
use unimock::{MockFn, Unimock, matching};

use crate::boundary::ClockMock;
use crate::error::Error;
use crate::types::Session;

use super::support::{at, gateway, id};

#[test]
fn rejects_unsupported_version_before_authentication() {
    let service = gateway(
        Unimock::new(()),
        Unimock::new(()),
        Unimock::new(()),
        Unimock::new(()),
        Unimock::new(()),
        Unimock::new(()),
    );
    let message: AgentMessage =
        serde_json::from_str(include_str!("unsupported_hello.json")).unwrap();
    assert!(matches!(
        service.open(message),
        Err(Error::UnsupportedVersion)
    ));
}

#[test]
fn rejects_a_report_whose_platform_differs_from_the_registration() {
    let session = Session {
        workspace_id: id(1),
        machine_id: id(3),
        id: id(4),
    };
    let service = gateway(
        Unimock::new((
            GatewayStoreMock::machine_session
                .each_call(matching!(_, _))
                .answers(&|_, _, _| Ok(Some(id(4)))),
            GatewayStoreMock::record_contact
                .some_call(matching!(_, _, _))
                .answers(&|_, _, _, _| Err(StoreError::InvalidValue)),
        )),
        Unimock::new(()),
        Unimock::new(ClockMock::now.each_call(matching!()).returns(at())),
        Unimock::new(()),
        Unimock::new(()),
        Unimock::new(()),
    );
    let report = serde_json::from_value(serde_json::json!({
        "hostname": "builder", "agent_version": "0.1.0",
        "hardware": {"platform": "macos_arm64", "processor": "Apple M4", "threads": 10,
            "memory_bytes": 16_000_000_000u64, "virtualization": "available", "disks": []},
        "capacity": 1, "job_disk_bytes": 0, "slots": [], "runners": [], "drain": "active",
        "cache": {"used_bytes": 0, "capacity_bytes": 0, "restores_last_7_days": 0,
            "hits_last_7_days": 0, "jobs": []},
        "docker_disks": {"leased_bytes": 0, "repositories": 0, "used_bytes": 0},
        "images": {"available": [], "newest": null}
    }))
    .unwrap();
    assert!(matches!(
        service.receive(session, AgentMessage::Report { report }),
        Err(Error::InvalidMessage)
    ));
}
