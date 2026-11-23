//! Construct the overflow service with KMS and the deployed provider images.

use std::sync::Arc;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_github::registrations::RunnerRegistrations;
use ciproxy_github::runtime::SystemRuntime;
use ciproxy_overflow::api::{Overflow, OverflowConfig};
use ciproxy_overflow::boat::BoatFactory;
use ciproxy_overflow::clock::SystemClock;
use ciproxy_overflow::kms::cipher::CloudKmsCipher;
use ciproxy_overflow::kms::transport::{CloudKmsRest, MetadataAccessTokens};
use ciproxy_overflow::registry::{ImageSource, ProviderRegistry};
use ciproxy_overflow::secrets::RandomTokens;
use ciproxy_overflow::service::{OverflowDependencies, OverflowService};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::store::Store;

use crate::config::Config;
use crate::error::{ErrorResultExt, Result};
use crate::overflow_authority::{OverflowRegistrationRetirement, OverflowRunnerAuthority};
use crate::overflow_callback::EXIT_PATH;

pub(crate) struct OverflowRuntime {
    pub(crate) service: Arc<dyn Overflow>,
    pub(crate) retirement: Arc<dyn OverflowRegistrationRetirement>,
    pub(crate) providers: Arc<ProviderRegistry>,
}

pub(crate) fn compose(
    config: &Config,
    store: Arc<dyn Store>,
    registrations: Arc<dyn RunnerRegistrations>,
) -> Result<OverflowRuntime> {
    let root_sizing = RootSizing {
        image_root_bytes: config.image_disk_bytes,
    };
    let providers = Arc::new(
        ProviderRegistry::default().with_provider(
            OverflowProvider::Boat,
            Arc::new(BoatFactory::new()),
            config
                .boat_shared_snapshot
                .as_ref()
                .map(|snapshot| ImageSource {
                    reference: snapshot.clone(),
                    root_sizing,
                }),
        ),
    );
    let transport = Arc::new(
        CloudKmsRest::new(
            config.overflow_kms_key.clone(),
            Arc::new(MetadataAccessTokens::new().internal()?),
        )
        .internal()?,
    );
    let retirement = Arc::new(OverflowRunnerAuthority {
        store: store.clone(),
        registrations,
        root_sizing,
        providers: providers.clone(),
        clock: Arc::new(SystemRuntime),
    });
    let deployment = config.public_url.trim_end_matches('/').to_owned();
    let service = Arc::new(OverflowService::new(
        OverflowConfig {
            exit_report_url: Secret::new(format!("{deployment}{EXIT_PATH}")),
            deployment,
        },
        OverflowDependencies {
            store,
            cipher: Arc::new(CloudKmsCipher::new(transport)),
            providers: providers.clone(),
            runners: retirement.clone(),
            clock: Arc::new(SystemClock),
            tokens: Arc::new(RandomTokens),
        },
    ));
    Ok(OverflowRuntime {
        service,
        retirement,
        providers,
    })
}
