//! Wallet preselection follows Boat's active choice before its first offer.

use ciproxy_overflow::provider::{BillingAccount, KeyAccounts};
use serde_json::json;

use crate::tls::facts::identity;
use crate::tls::peer::step;
use crate::tls::server::Server;

#[tokio::test]
async fn wallets_preselects_the_active_wallet_or_first_offer() {
    for (case, orgs, expected) in [
        (
            "later active organization",
            json!([
                {"id":"user_customer","name":"Personal","type":"personal","active":false},
                {"id":"team_customer","name":"Customer Organization","type":"org","active":true}
            ]),
            KeyAccounts {
                accounts: vec![
                    BillingAccount {
                        id: "personal".to_owned(),
                        name: "Customer Boat".to_owned(),
                    },
                    BillingAccount {
                        id: "team_customer".to_owned(),
                        name: "Customer Organization".to_owned(),
                    },
                ],
                selected_account: "team_customer".to_owned(),
            },
        ),
        (
            "no active wallet",
            json!([
                {"id":"user_customer","name":"Personal","type":"personal","active":false},
                {"id":"team_customer","name":"Customer Organization","type":"org","active":false}
            ]),
            KeyAccounts {
                accounts: vec![
                    BillingAccount {
                        id: "personal".to_owned(),
                        name: "Customer Boat".to_owned(),
                    },
                    BillingAccount {
                        id: "team_customer".to_owned(),
                        name: "Customer Organization".to_owned(),
                    },
                ],
                selected_account: "personal".to_owned(),
            },
        ),
    ] {
        let server = Server::new(vec![
            step("GET", "/api/v1/me", 200, identity()).wallet(None),
            step(
                "GET",
                "/api/v1/orgs",
                200,
                json!({"ok":true,"type":"org.list","orgs":orgs}),
            )
            .wallet(None),
        ])
        .await;
        let result = server.backend.account.check_key().await;
        assert_eq!(result.as_ref().ok(), Some(&expected), "{case}: {result:?}");
        server.assert_consumed();
    }
}
