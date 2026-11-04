//! Shared public-API assertions for named message snapshots.

use std::fmt::Debug;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;

pub(super) fn snapshot<T>(name: &str, tag: &str, message: T)
where
    T: Serialize + DeserializeOwned + Debug + PartialEq,
{
    let value = serde_json::to_value(&message).unwrap();
    assert_eq!(value["type"], json!(tag));
    assert_eq!(serde_json::from_value::<T>(value).unwrap(), message);
    insta::with_settings!({
        prepend_module_to_snapshot => false,
        description => format!("Tagged JSON for {name}; all credentials are dummy test values."),
    }, {
        insta::assert_json_snapshot!(name, message);
    });
}
