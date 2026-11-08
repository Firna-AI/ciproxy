//! Wallet preselection follows Boat's active choice before its first offer.

use ciproxy_overflow::provider::{AccountWallet, KeyWallets};
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
            KeyWallets {
                wallets: vec![
                    AccountWallet {
                        wallet: "personal".to_owned(),
                        name: "Customer Boat".to_owned(),
                    },
                    AccountWallet {
                        wallet: "team_customer".to_owned(),
                        name: "Customer Organization".to_owned(),
                    },
                ],
                selected_wallet: "team_customer".to_owned(),
            },
        ),
        (
            "no active wallet",
            json!([
                {"id":"user_customer","name":"Personal","type":"personal","active":false},
                {"id":"team_customer","name":"Customer Organization","type":"org","active":false}
            ]),
            KeyWallets {
                wallets: vec![
                    AccountWallet {
                        wallet: "personal".to_owned(),
                        name: "Customer Boat".to_owned(),
                    },
                    AccountWallet {
                        wallet: "team_customer".to_owned(),
                        name: "Customer Organization".to_owned(),
                    },
                ],
                selected_wallet: "personal".to_owned(),
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
        let result = server.backend.account.wallets().await;
        assert_eq!(result.as_ref().ok(), Some(&expected), "{case}: {result:?}");
        server.assert_consumed();
    }
}
