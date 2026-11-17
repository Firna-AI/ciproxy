//! App JWTs retain the configured issuer and clock-relative RSA-signed claims.

use std::sync::Arc;

use chrono::DateTime;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use secrecy::ExposeSecret;
use serde::Deserialize;
use unimock::{MockFn, Unimock, matching};

use crate::auth::AppJwt;
use crate::auth::jwt::RsaAppJwt;
use crate::runtime::RuntimeMock;

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Claims {
    iss: String,
    iat: i64,
    exp: i64,
}

#[test]
fn rsa_app_jwt_signs_current_issuer_and_bounded_times_after_clock_advances() {
    let runtime = Arc::new(Unimock::new(RuntimeMock::now.stub(|each| {
        each.call(matching!())
            .returns(DateTime::from_timestamp(1_793_793_600, 0).unwrap())
            .once()
            .then()
            .returns(DateTime::from_timestamp(1_793_794_200, 0).unwrap())
            .once();
    })));
    let signer = RsaAppJwt::from_pem(
        "test-app-client-id".to_owned(),
        include_bytes!("fixtures/app-test-private-key.pem"),
        runtime,
    )
    .unwrap();
    let public_key =
        DecodingKey::from_rsa_pem(include_bytes!("fixtures/app-test-public-key.pem")).unwrap();
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = false;
    for (issued, expires) in [
        (1_793_793_540, 1_793_794_080),
        (1_793_794_140, 1_793_794_680),
    ] {
        let token = signer.token().unwrap();
        let decoded = decode::<Claims>(token.expose_secret(), &public_key, &validation);
        assert!(decoded.is_ok(), "the App public key must verify the JWT");
        let decoded = decoded.unwrap();
        assert_eq!(decoded.header.alg, Algorithm::RS256);
        assert_eq!(
            decoded.claims,
            Claims {
                iss: "test-app-client-id".to_owned(),
                iat: issued,
                exp: expires,
            }
        );
    }
}
