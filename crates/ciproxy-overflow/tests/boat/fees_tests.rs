//! Published start fees take precedence over a changing next-start quote.

use rust_decimal::Decimal;
use serde_json::json;

use crate::tls::facts::{identity, limits};
use crate::tls::peer::step;
use crate::tls::server::Server;

#[tokio::test]
async fn account_uses_the_highest_bracket_and_only_falls_back_without_brackets() {
    let mut maximum = limits(true);
    maximum["nextStartDollars"] = json!(0);
    let mut missing = limits(true);
    missing.as_object_mut().unwrap().remove("overagePrices");
    let mut empty = limits(true);
    empty["overagePrices"]["startBrackets"] = json!([]);
    let mut neither = missing.clone();
    neither.as_object_mut().unwrap().remove("nextStartDollars");
    for (response, expected) in [
        (maximum, Some(Decimal::new(7, 2))),
        (missing, Some(Decimal::new(3, 2))),
        (empty, Some(Decimal::new(3, 2))),
        (neither, None),
    ] {
        let server = Server::new(vec![
            step("GET", "/api/v1/me", 200, identity()),
            step("GET", "/api/v1/limits", 200, response),
        ])
        .await;
        let facts = server.backend.account.account().await.unwrap();
        assert_eq!(facts.start_fee_bound_usd, expected);
        assert_eq!(facts.billing_ready, expected.is_some());
        server.assert_consumed();
        let captured = server.captured();
        assert_eq!(captured.len(), 2);
    }
}
