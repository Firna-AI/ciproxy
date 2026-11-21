//! Polling keeps other installations fresh after one provider failure.

use std::sync::{Arc, Mutex};

use ciproxy_github::app::GitHubAppMock;
use unimock::{MockFn, Unimock, matching};

use super::{InstallationsMock, RefreshRepositories, RepositoryRefresh};

#[tokio::test]
async fn metadata_polling_retries_each_retained_installation_independently() {
    let refreshed = Arc::new(Mutex::new(Vec::new()));
    let observed = refreshed.clone();
    let service = RefreshRepositories {
        installations: Arc::new(Unimock::new(
            InstallationsMock::retained
                .some_call(matching!())
                .returns(Ok(vec![41, 42])),
        )),
        app: Arc::new(Unimock::new(
            GitHubAppMock::refresh_repositories
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, id| {
                    observed.lock().unwrap().push(id);
                    if id == 41 {
                        Err(ciproxy_github::error::Error::AccessUnavailable(503))
                    } else {
                        Ok(())
                    }
                })),
        )),
    };
    assert_eq!(
        service
            .refresh()
            .await
            .map_err(|_| "another installation blocked refresh"),
        Ok(())
    );
    assert_eq!(*refreshed.lock().unwrap(), vec![41, 42]);
}
