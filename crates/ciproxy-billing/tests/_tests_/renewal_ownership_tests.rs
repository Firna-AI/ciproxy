//! A mismatched current subscription cannot change durable renewal evidence.

use ciproxy_billing::error::Error;
use serde_json::json;

use super::fake::Fake;
use super::live_replay::{Recording, deliver, install, saved, send};

#[test]
fn unowned_current_subscription_preserves_coverage_and_renewal_evidence() {
    for variation in ["customer", "workspace"] {
        let recording = Recording::recovery();
        let database = recording.seed();
        let id = recording.initial_state.subscription_id.as_deref().unwrap();
        let invoice_id = recording.events[0]["data"]["object"]["id"]
            .as_str()
            .unwrap();
        let current = recording.snapshot(id, "past_due");
        let fake = Fake::new(serde_json::from_value(current.clone()).unwrap());
        install(
            &fake,
            current.clone(),
            &[recording.snapshot(invoice_id, "open")],
        );
        deliver(&database, &fake, &recording.events[0]);
        let before = saved(&database, &recording);
        let before_renewals = database
            .store
            .try_lock_billing(before.workspace_id)
            .unwrap()
            .unwrap()
            .renewals(id)
            .unwrap();
        let mut invalid = current.clone();
        match variation {
            "customer" => invalid["customer"] = json!("cus_derived_other"),
            "workspace" => {
                invalid["metadata"]["workspace_id"] = json!("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa")
            }
            _ => unreachable!(),
        }
        install(&fake, invalid, &[recording.snapshot(invoice_id, "paid")]);
        assert!(
            matches!(
                send(&database, &fake, &recording.events[1]),
                Err(Error::Ownership)
            ),
            "{variation}"
        );
        assert_eq!(saved(&database, &recording), before, "{variation}");
        let after_renewals = database
            .store
            .try_lock_billing(before.workspace_id)
            .unwrap()
            .unwrap()
            .renewals(id)
            .unwrap();
        assert_eq!(after_renewals, before_renewals, "{variation}");
    }
}
