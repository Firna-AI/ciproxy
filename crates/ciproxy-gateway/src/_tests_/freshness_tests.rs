//! Exact offline boundary without a timer or network.

use chrono::Duration;
use ciproxy_store_interface::gateway::GatewayStoreMock;
use unimock::{MockFn, Unimock, matching};

use crate::boundary::ClockMock;
use crate::types::Presence;

use super::support::{at, gateway, id, snapshot};

#[test]
fn marks_health_old_at_sixty_seconds_and_retains_last_contact() {
    for (seconds, presence) in [(59, Presence::Online), (60, Presence::Offline)] {
        let service = gateway(
            Unimock::new(
                GatewayStoreMock::machine_snapshots
                    .some_call(matching!((workspace) if *workspace == id(1)))
                    .returns(Ok(vec![snapshot()])),
            ),
            Unimock::new(()),
            Unimock::new(
                ClockMock::now
                    .some_call(matching!())
                    .returns(at() + Duration::seconds(seconds)),
            ),
            Unimock::new(()),
            Unimock::new(()),
            Unimock::new(()),
        );
        let actual = service.machines(id(1)).unwrap();
        assert_eq!(actual[0].presence, presence, "at {seconds} seconds");
        assert_eq!(actual[0].snapshot.machine.last_contact_at, Some(at()));
    }
}
