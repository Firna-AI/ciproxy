//! Literal Stripe preview forms and response fixtures for the composed route.

use serde_json::json;

use super::fake::Fake;

pub(super) fn previews(fake: &Fake, upgrade: bool, monthly: i64, items: &[(&str, &str)]) {
    let mut common = vec![
        ("customer".to_owned(), "cus_recorded".to_owned()),
        ("subscription".to_owned(), "sub_recorded".to_owned()),
    ];
    common.extend(
        items
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string())),
    );
    let mut immediate = common.clone();
    immediate.extend([
        ("preview_mode".into(), "next".into()),
        (
            "subscription_details[proration_date]".into(),
            "1804334400".into(),
        ),
        (
            "subscription_details[proration_behavior]".into(),
            if upgrade {
                "always_invoice"
            } else {
                "create_prorations"
            }
            .into(),
        ),
    ]);
    common.push(("preview_mode".into(), "recurring".into()));
    let mut remote = fake.state.lock().unwrap();
    for (fields, total, due) in [
        (
            immediate,
            if upgrade { 17431 } else { -17431 },
            if upgrade { 17431 } else { 0 },
        ),
        (common, monthly, monthly),
    ] {
        remote.previews.push_back((fields, json!({
            "currency":"usd", "customer":"cus_recorded", "parent":{"subscription_details":{"subscription":"sub_recorded"}},
            "amount_due":due, "total":total,"starting_balance":0,
        })));
    }
}

pub(super) const UPGRADE: &[(&str, &str)] = &[
    ("subscription_details[items][0][id]", "si_tier"),
    ("subscription_details[items][0][deleted]", "true"),
    ("subscription_details[items][1][id]", "si_starts"),
    ("subscription_details[items][1][price]", "price_starts"),
    ("subscription_details[items][1][quantity]", "3"),
    ("subscription_details[items][2][id]", "si_machine"),
    ("subscription_details[items][2][deleted]", "true"),
    ("subscription_details[items][3][price]", "price_business"),
    ("subscription_details[items][3][quantity]", "1"),
];

pub(super) const DOWNGRADE: &[(&str, &str)] = &[
    ("subscription_details[items][0][id]", "si_business"),
    ("subscription_details[items][0][deleted]", "true"),
    ("subscription_details[items][1][id]", "si_starts"),
    ("subscription_details[items][1][price]", "price_starts"),
    ("subscription_details[items][1][quantity]", "2"),
    ("subscription_details[items][2][price]", "price_team"),
    ("subscription_details[items][2][quantity]", "1"),
];
