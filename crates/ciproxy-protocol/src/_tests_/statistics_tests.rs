//! Statistics distinguish missing measurements from measured legacy zeros.

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::statistics::Statistics;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Figures {
    used_bytes: u64,
}

#[test]
fn legacy_and_current_measured_statistics_keep_true_zeros() {
    for wire in [
        json!({"used_bytes": 0}),
        json!({"status": "measured", "used_bytes": 0}),
    ] {
        let value: Statistics<Figures> = serde_json::from_value(wire).unwrap();
        assert_eq!(value, Statistics::Measured(Figures { used_bytes: 0 }));
        assert_eq!(
            serde_json::to_value(&value).unwrap(),
            json!({"status": "measured", "used_bytes": 0})
        );
    }
}

#[test]
fn unavailable_statistics_never_turn_into_legacy_figures() {
    for wire in [
        json!({"status": "unavailable"}),
        json!({"status": "unavailable", "used_bytes": 900}),
    ] {
        let value: Statistics<Figures> = serde_json::from_value(wire).unwrap();
        assert_eq!(value, Statistics::Unavailable);
        assert_eq!(
            serde_json::to_value(value).unwrap(),
            json!({"status": "unavailable"})
        );
    }
}
