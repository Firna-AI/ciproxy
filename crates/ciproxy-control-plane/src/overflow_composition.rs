//! Construct the shared Boat executor with KMS and the deployed image release.

use std::sync::Arc;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_github::registrations::RunnerRegistrations;
use ciproxy_github::runtime::SystemRuntime;
use ciproxy_overflow::api::{Overflow, OverflowConfig};
use ciproxy_overflow::boat::BoatFactory;
use ciproxy_overflow::clock::SystemClock;
use ciproxy_overflow::kms::cipher::CloudKmsCipher;
use ciproxy_overflow::kms::transport::{CloudKmsRest, MetadataAccessTokens};
use ciproxy_overflow::secrets::RandomTokens;
use ciproxy_overflow::service::{OverflowDependencies, OverflowService};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::store::Store;

use crate::boat_authority::{BoatRegistrationRetirement, BoatRunnerAuthority};
use crate::config::Config;
use crate::error::{ErrorResultExt, Result};

pub(crate) struct OverflowRuntime {
    pub(crate) service: Arc<dyn Overflow>,
    pub(crate) retirement: Arc<dyn BoatRegistrationRetirement>,
    pub(crate) root_sizing: RootSizing,
}

pub(crate) fn compose(
    config: &Config,
    store: Arc<dyn Store>,
    registrations: Arc<dyn RunnerRegistrations>,
) -> Result<Option<OverflowRuntime>> {
    let Some(snapshot) = &config.boat_shared_snapshot else {
        return Ok(None);
    };
    let root_sizing = RootSizing {
        image_root_bytes: config.image_disk_bytes,
    };
    let transport = Arc::new(
        CloudKmsRest::new(
            config.boat_kms_key.clone(),
            Arc::new(MetadataAccessTokens::new().internal()?),
        )
        .internal()?,
    );
    let retirement = Arc::new(BoatRunnerAuthority {
        store: store.clone(),
        registrations,
        root_sizing,
        clock: Arc::new(SystemRuntime),
    });
    let service = Arc::new(OverflowService::new(
        OverflowConfig {
            deployment: config.public_url.trim_end_matches('/').to_owned(),
            shared_snapshot: snapshot.clone(),
            root_sizing,
            exit_report_url: Secret::new(format!(
                "{}/hooks/boat/runner-exit",
                config.public_url.trim_end_matches('/')
            )),
        },
        OverflowDependencies {
            store: store.clone(),
            cipher: Arc::new(CloudKmsCipher::new(transport)),
            factory: Arc::new(BoatFactory::new()),
            runners: retirement.clone(),
            clock: Arc::new(SystemClock),
            tokens: Arc::new(RandomTokens),
        },
    ));
    Ok(Some(OverflowRuntime {
        service,
        retirement,
        root_sizing,
    }))
}
