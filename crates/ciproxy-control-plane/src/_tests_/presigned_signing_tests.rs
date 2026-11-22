//! The presigned URL matches the published AWS example and hides its key.

use std::time::Duration;

use super::{PresignedSigning, StoreKey};
use crate::error::Error;
use crate::image_signing::ImageSigning;

const AWS_EXAMPLE_KEY: &str = r#"{"access_key_id": "AKIAIOSFODNN7EXAMPLE", "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"}"#;

#[test]
fn presigned_get_matches_the_published_aws_signature_example() {
    let key = StoreKey::from_json("R2_IMAGES_KEY", AWS_EXAMPLE_KEY).unwrap();
    let signer = PresignedSigning::new(key, "us-east-1", Duration::from_secs(86_400));
    let at = "2013-05-24T00:00:00Z".parse().unwrap();
    let signed = signer
        .sign("https://examplebucket.s3.amazonaws.com/test.txt", at)
        .unwrap();
    assert_eq!(
        signed.expose_secret(),
        "https://examplebucket.s3.amazonaws.com/test.txt\
         ?X-Amz-Algorithm=AWS4-HMAC-SHA256\
         &X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request\
         &X-Amz-Date=20130524T000000Z\
         &X-Amz-Expires=86400\
         &X-Amz-SignedHeaders=host\
         &X-Amz-Signature=aeeed9bbccd4d02ee5c0109b86d86835f995330da4c265957d157751f604d404"
    );
}

#[test]
fn debug_output_shows_no_part_of_the_key() {
    let key = StoreKey::from_json("R2_IMAGES_KEY", AWS_EXAMPLE_KEY).unwrap();
    let signer = PresignedSigning::new(key.clone(), "auto", Duration::from_secs(300));
    for output in [format!("{key:?}"), format!("{signer:?}")] {
        assert!(!output.contains("AKIAIOSFODNN7EXAMPLE"), "{output}");
        assert!(!output.contains("wJalrXUtnFEMI"), "{output}");
    }
}

#[test]
fn a_malformed_key_names_the_setting_and_not_the_value() {
    for json in [
        r#""wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY""#,
        r#"{"access_key_id": "AKIAIOSFODNN7EXAMPLE"}"#,
        r#"{"access_key_id": "", "secret_access_key": "wJalrXUtnFEMI"}"#,
    ] {
        let error = StoreKey::from_json("R2_IMAGES_KEY", json).unwrap_err();
        assert!(
            matches!(error, Error::MalformedSetting("R2_IMAGES_KEY")),
            "{json}"
        );
        assert_eq!(
            error.to_string(),
            "[control-plane] malformed setting R2_IMAGES_KEY"
        );
    }
}
