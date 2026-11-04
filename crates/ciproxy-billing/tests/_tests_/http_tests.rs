//! HTTP catalog provisioning and restricted customer portal contracts.

use ciproxy_billing::stripe::api::StripeApi;
use ciproxy_billing::stripe::http::client::HttpStripe;

use super::fake::{Fake, field};
use super::fixture::subscription;

#[test]
fn catalog_creates_four_usd_monthly_prices_and_portal_blocks_item_edits() {
    let fake = Fake::new(subscription());
    fake.state.lock().unwrap().create_prices = true;
    let provider = HttpStripe::with_base_url("sk_test_redaction_marker".into(), &fake.url).unwrap();
    let provider_api: &dyn StripeApi = &provider;
    provider_api.provision().unwrap();
    provider_api.provision().unwrap();
    let prices = fake.posts("/v1/prices");
    assert_eq!(prices.len(), 4);
    for (request, amount) in prices.iter().zip(["4900", "29900", "1000", "2000"]) {
        assert_eq!(field(request, "unit_amount"), amount);
        assert_eq!(field(request, "currency"), "usd");
        assert_eq!(field(request, "recurring[interval]"), "month");
    }
    assert_eq!(
        provider_api
            .portal("cus_recorded", "https://app.example/billing")
            .unwrap(),
        "https://billing.stripe.test/portal"
    );
    let configurations = fake.posts("/v1/billing_portal/configurations");
    assert_eq!(
        field(&configurations[0], "features[invoice_history][enabled]"),
        "true"
    );
    assert_eq!(
        field(
            &configurations[0],
            "features[payment_method_update][enabled]"
        ),
        "true"
    );
    assert_eq!(
        field(&configurations[0], "features[subscription_cancel][mode]"),
        "at_period_end"
    );
    assert_eq!(
        field(&configurations[0], "features[subscription_update][enabled]"),
        "false"
    );
    assert_eq!(
        field(
            &fake.posts("/v1/billing_portal/sessions")[0],
            "configuration"
        ),
        "bpc_restricted"
    );
    assert_eq!(
        format!("{provider:?}"),
        "HttpStripe { secret_key: \"[redacted]\", .. }"
    );
}
