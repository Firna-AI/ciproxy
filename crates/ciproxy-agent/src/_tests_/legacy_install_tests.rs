//! Install reads retained routes without stopping service or changing identity.

use std::sync::Arc;

use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::application_endpoint::ConfiguredAction;
use crate::application_metadata::MetadataTarget;
use crate::application_setup::InvocationSetup;
use crate::boundaries::{ConnectionMock, HostMock, ServiceMock};
use crate::cli::Action;
use crate::identity_store_fixtures::{IdentityFiles, legacy_state};
use crate::installation::{Install, Installer};
use crate::preparation::PreparationMock;

#[tokio::test]
async fn legacy_repair_keeps_service_and_state_and_reports_the_retained_endpoint() {
    let files = IdentityFiles::new(legacy_state());
    let prepared = files
        .setup()
        .prepare(Action::Install {
            token: Secret::new("synthetic-unused-install-token"),
            agent_url: None,
        })
        .unwrap();
    assert_eq!(*files.effects.lock().unwrap(), Vec::<&str>::new());
    let metadata_url = match MetadataTarget::for_machine(prepared.state.identity.as_ref()) {
        MetadataTarget::Hosted(metadata) => Some(metadata.url.expose_secret().to_owned()),
        MetadataTarget::Disabled | MetadataTarget::Unavailable => None,
    };
    assert_eq!(
        metadata_url.as_deref(),
        Some("https://preview.ciproxy.example/agent/cache-metadata")
    );
    let ConfiguredAction::Install { endpoint, token } = prepared.action else {
        panic!("repair must keep the install action");
    };
    let connects = files.effects.clone();
    let sends = files.effects.clone();
    let mock = Arc::new(
        Unimock::new((
            HostMock::require_root
                .some_call(matching!())
                .returns(Ok(())),
            PreparationMock::prepare
                .some_call(matching!())
                .returns(Ok(())),
            ServiceMock::install.some_call(matching!()).returns(Ok(())),
            ConnectionMock::connect
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    connects.lock().unwrap().push("connect");
                    Ok(())
                })),
            ConnectionMock::send
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, _| {
                    sends.lock().unwrap().push("send");
                    Ok(())
                })),
        ))
        .no_verify_in_drop(),
    );
    let installer = Installer {
        endpoint,
        store: files.store(),
        host: mock.clone(),
        service: mock.clone(),
        connection: mock.clone(),
        agent: mock.clone(),
        preparation: mock,
    };
    let message = installer.install(token).await.unwrap().message();
    assert_eq!(
        message.lines().last(),
        Some(
            "ciproxy agent repaired. Machine stays at wss://preview.ciproxy.example/agent/. Command selected wss://app.ciproxy.dev/agent/. To move the machine, uninstall first."
        )
    );
    assert_eq!(*files.state.lock().unwrap(), legacy_state());
    assert_eq!(*files.effects.lock().unwrap(), Vec::<&str>::new());
    assert_eq!(
        files.legacy.lock().unwrap().as_deref(),
        Some("wss://preview.ciproxy.example/agent/")
    );
}
