//! Admin-only actions and hash-only token issuance.

use chrono::Duration;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::gateway::GatewayStoreMock;
use unimock::{MockFn, Unimock, matching};

use crate::boundary::{AuthorizationMock, ClockMock, EntropyMock};
use crate::error::Error;
use crate::types::MachineAction;

use super::support::{at, gateway, id};

#[test]
fn mints_hash_only_one_hour_token_for_workspace_admin() {
    const EXPECTED_HASH: &[u8] = &[
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22,
        0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00,
        0x15, 0xad,
    ];
    let store = Unimock::new(
        GatewayStoreMock::issue_install_token
            .some_call(
                matching!((workspace, token, hash, time) if *workspace == id(1) && *token == id(8)
            && hash.as_bytes() == EXPECTED_HASH && *time == at()),
            )
            .returns(Ok(())),
    );
    let service = gateway(
        store,
        Unimock::new(
            AuthorizationMock::require_admin
                .some_call(matching!((principal, workspace)
            if *principal == id(7) && *workspace == id(1)))
                .returns(Ok(())),
        ),
        Unimock::new(ClockMock::now.some_call(matching!()).returns(at())),
        Unimock::new((
            EntropyMock::identifier
                .some_call(matching!())
                .returns(Ok(id(8))),
            EntropyMock::secret
                .some_call(matching!())
                .returns(Ok(Secret::new("abc"))),
        )),
        Unimock::new(()),
        Unimock::new(()),
    );
    let issued = service.issue_token(id(7), id(1)).unwrap();
    assert_eq!(issued.expires_at, at() + Duration::hours(1));
    assert_eq!(issued.id, id(8));
}

#[test]
fn denies_token_and_machine_management_without_admin_authority() {
    for operation in 0..6 {
        let service = gateway(
            Unimock::new(()),
            Unimock::new(
                AuthorizationMock::require_admin
                    .some_call(matching!((principal, workspace)
                if *principal == id(7) && *workspace == id(2)))
                    .returns(Err(Error::Forbidden)),
            ),
            Unimock::new(()),
            Unimock::new(()),
            Unimock::new(()),
            Unimock::new(()),
        );
        let result = match operation {
            0 => service.issue_token(id(7), id(2)).map(|_| ()),
            1 => service.revoke_token(id(7), id(2), id(9)),
            2 => service
                .manage(id(7), id(2), id(3), MachineAction::Drain)
                .map(|_| ()),
            3 => service
                .manage(id(7), id(2), id(3), MachineAction::Resume)
                .map(|_| ()),
            4 => service
                .manage(id(7), id(2), id(3), MachineAction::Remove)
                .map(|_| ()),
            _ => service
                .manage(id(7), id(2), id(3), MachineAction::ClearCache)
                .map(|_| ()),
        };
        assert!(
            matches!(result, Err(Error::Forbidden)),
            "operation {operation}"
        );
    }
}

#[test]
fn internal_diagnostics_do_not_expose_original_secrets() {
    let error = Error::internal_message("dummy-private-install-token");
    assert_eq!(format!("{error:?}"), "[gateway] internal operation failed");
    assert_eq!(format!("{error}"), "[gateway] internal operation failed");
}
