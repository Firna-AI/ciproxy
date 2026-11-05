//! Account fixtures with unsorted published start fees.

use serde_json::{Value, json};

pub(crate) fn limits(can_start: bool) -> Value {
    json!({"ok":true,"type":"limits.info","canStart":can_start,"billingStatus":"active","sandboxPlanKey":"box_20", "hasPaymentHistory":true,"nextStartDollars":0.03, "overagePrices":{"startBrackets":[{"dollars":0.03},{"dollars":0.07},{"dollars":0.05}]},
        "machineTypes":[{"type":"large","available":true,"reason":null}]})
}
pub(crate) fn identity() -> Value {
    json!({"ok":true,"type":"user.info","user":{"login":"customer","displayName":"Customer Boat"}})
}
