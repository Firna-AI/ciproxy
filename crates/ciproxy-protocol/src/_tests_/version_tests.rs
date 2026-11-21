//! Wire compatibility of the shared version type.

use serde_json::json;

use super::ProtocolVersion;

#[test]
fn current_version_uses_an_integer_on_the_wire() {
    assert_eq!(
        serde_json::to_value(ProtocolVersion::CURRENT).unwrap(),
        json!(1)
    );
    assert_eq!(
        serde_json::from_value::<ProtocolVersion>(json!(1)).unwrap(),
        ProtocolVersion::CURRENT
    );
}
