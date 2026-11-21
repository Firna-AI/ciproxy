//! A later browser return cannot change an unregistered signed flow.

use ciproxy_store_interface::installation_association::SignedInstallation;
use ciproxy_store_interface::installation_links::{InstallationAttempt, InstallationRecovery};
use ciproxy_store_interface::installation_results::InstallationResultDelivery;
use uuid::Uuid;

use super::super::support::{Database, at};

#[test]
fn signed_direct_flow_survives_later_add_proof_and_replay_before_registration() {
    let database = Database::new();
    let signed = SignedInstallation {
        id: Uuid::new_v4(),
        installation_id: 1066,
        github_id: 106,
        login: "signed-org".to_owned(),
        requester_id: 7,
    };
    let mut claim = database
        .store
        .associate_signed_installation(&signed)
        .unwrap();
    assert_eq!(claim.recovery(), InstallationRecovery::Direct);
    assert_eq!(claim.organization_id, None);
    let attempt = InstallationAttempt {
        id: Uuid::new_v4(),
        state_digest: Uuid::new_v4().as_bytes().to_vec(),
        requester_id: 7,
        direct: false,
        installation_id: None,
    };
    database.store.begin_installation_attempt(&attempt).unwrap();
    claim.event_seen = false;
    claim.direct = false;
    claim.user_verified = true;
    claim.verified_at = Some(at());
    let proof = database
        .store
        .save_installation_claim(
            &claim,
            Some(attempt.id),
            InstallationResultDelivery::Untracked,
        )
        .unwrap();
    assert_eq!(
        proof.recovery(),
        InstallationRecovery::Direct,
        "fresh Add proof cannot change a signed Direct flow before registration"
    );
    let replay = database
        .store
        .associate_signed_installation(&signed)
        .unwrap();
    assert_eq!(
        replay.recovery(),
        InstallationRecovery::Direct,
        "signed replay must keep its first flow despite later exact Add evidence"
    );
    assert_eq!(replay.id, signed.id);
    assert_eq!(replay.organization_id, None);
    assert_eq!(replay.workspace_id, None);
}
