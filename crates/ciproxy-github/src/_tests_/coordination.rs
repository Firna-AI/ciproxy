//! Literal installation snapshots and mock mutation authority.

use std::sync::Arc;

use unimock::{Clause, MockFn, matching};

use crate::installation_state::{InstallationApiMock, InstalledApp};
use crate::persistence::mutation::{MutationLease, MutationLeaseMock, OrganizationMutationsMock};

use super::support::event;
use crate::app::webhook::InstallationAction;

pub(super) fn installed(id: i64, suspended: bool) -> InstalledApp {
    InstalledApp {
        id,
        account: event(InstallationAction::Created).installation.account,
        suspended,
    }
}

pub(super) fn lease() -> impl Clause {
    (
        OrganizationMutationsMock::acquire
            .each_call(matching!(17))
            .answers(&|mock, _| Ok(Box::new(mock.clone()) as Box<dyn MutationLease>)),
        MutationLeaseMock::check
            .each_call(matching!())
            .answers(&|_| Ok(())),
    )
}

pub(super) fn coordination(suspended: bool) -> impl Clause {
    (
        lease(),
        InstallationApiMock::for_organization
            .each_call(matching!("customer"))
            .answers_arc(Arc::new(move |_, _| Ok(installed(19, suspended)))),
    )
}
