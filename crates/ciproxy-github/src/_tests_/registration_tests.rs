//! JIT delivery and typed busy/removed outcomes preserve running work.

use std::sync::Arc;

use ciproxy_store_interface::organizations::AppState;
use secrecy::ExposeSecret;
use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::persistence::session::SessionStoreMock;
use crate::registrations::{RegistrationService, Removal, RunnerRegistrations};
use crate::scale_sets::{JitRegistration, ScaleSetApi, ScaleSetApiMock, ScaleSetFactoryMock};

use super::support::{context, target};

#[tokio::test]
async fn one_runner_registration_is_secret_and_busy_removal_keeps_job() {
    let mock = Arc::new(Unimock::new((
        SessionStoreMock::context
            .each_call(matching!(_))
            .answers(&|_, _| Ok(context())),
        ScaleSetFactoryMock::for_organization
            .each_call(matching!(_))
            .answers(&|mock, _| Ok(Arc::new(mock.clone()) as Arc<dyn ScaleSetApi>)),
        ScaleSetApiMock::register
            .some_call(matching!(53, "runner-71"))
            .returns(Ok(JitRegistration {
                runner_id: 71,
                configuration: "one-use-jit".into(),
            })),
        ScaleSetApiMock::remove_runner
            .some_call(matching!(71))
            .returns(Err(Error::RunnerBusy)),
    )));
    let service = RegistrationService {
        store: mock.clone(),
        scale_sets: mock,
    };
    let registration = service.create(target(), "runner-71").await.unwrap();
    assert_eq!(registration.runner_id, 71);
    assert_eq!(registration.configuration.expose_secret(), "one-use-jit");
    assert!(!format!("{registration:?}").contains("one-use-jit"));
    assert_eq!(service.remove(target(), 71).await.unwrap(), Removal::Busy);
}

#[tokio::test]
async fn removed_installation_keeps_inaccessible_github_registration() {
    let mut removed = context();
    removed.organization.app_state = AppState::Removed;
    let mock = Arc::new(Unimock::new(
        SessionStoreMock::context
            .some_call(matching!(_))
            .returns(Ok(removed)),
    ));
    let service = RegistrationService {
        store: mock.clone(),
        scale_sets: mock,
    };
    assert_eq!(
        service.remove(target(), 71).await.unwrap(),
        Removal::AccessGone
    );
}
