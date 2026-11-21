//! Default executable startup and bounded ownership of hosted background work.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use async_trait::async_trait;
use ciproxy_github::runtime::SystemRuntime;
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;
use reqwest::{Client, redirect};
use secrecy::ExposeSecret;
use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::watch;
use tokio::time::{MissedTickBehavior, interval, timeout};

use crate::config::Config;
use crate::error::{Error, ErrorResultExt, Result};
use crate::image_release::{CdnReleases, ImageReleases};
use crate::image_signing::CdnSigning;
use crate::startup_api::compose;
use crate::tasks::OwnedTask;

/// Executable boundary for the hosted process and its private configuration.
#[async_trait]
pub trait ControlPlane: Send + Sync {
    /// Resolve the pinned release and run the HTTP service until shutdown.
    async fn serve(&self, config: Config) -> Result<()>;
}

/// Production composition uses the existing mounted secrets and PostgreSQL store.
pub struct Production;

#[async_trait]
impl ControlPlane for Production {
    async fn serve(&self, mut config: Config) -> Result<()> {
        let mut terminate = signal(SignalKind::terminate()).internal()?;
        let starting = async {
            let releases: Arc<dyn ImageReleases> = Arc::new(CdnReleases {
                http: Client::builder()
                    .http1_only()
                    .redirect(redirect::Policy::none())
                    .connect_timeout(Duration::from_secs(5))
                    .timeout(Duration::from_secs(90))
                    .build()
                    .internal()?,
                prefix: config.image_url_prefix.clone(),
                manifest_path: config.image_manifest_path.clone(),
                signing: Arc::new(CdnSigning::new(
                    config.cdn_signing_key_name.clone(),
                    config.secret("cdn-signing-key")?.expose_secret(),
                )?),
                clock: Arc::new(SystemRuntime),
            });
            let release = releases.current().await?;
            config.image = Some(release.download);
            config.image_disk_bytes = release.root_disk_bytes;
            let database_url = config.database_url.expose_secret().to_owned();
            let store = tokio::task::spawn_blocking(move || PgStoreFactory.connect(&database_url))
                .await
                .internal()?
                .internal()?;
            let composition = compose(&config, store).await?;
            let listener = TcpListener::bind(config.http_bind).await.internal()?;
            Ok::<_, Error>((composition, listener))
        };
        let (composition, listener) = tokio::select! {
            result = starting => result?,
            _ = terminate.recv() => return Ok(()),
            result = tokio::signal::ctrl_c() => { result.internal()?; return Ok(()); },
        };
        let (shutdown, stop) = watch::channel(false);
        let hosted = composition.hosted.clone();
        let core_stop = stop.clone();
        let mut core = OwnedTask(tokio::spawn(
            async move { hosted.supervise(core_stop).await },
        ));
        let mut http_stop = stop.clone();
        let mut http = OwnedTask(tokio::spawn(async move {
            axum::serve(listener, composition.router)
                .with_graceful_shutdown(async move {
                    let _ = http_stop.changed().await;
                })
                .await
                .internal()
        }));
        let mut billing_stop = stop.clone();
        let mut billing = OwnedTask(tokio::spawn(async move {
            let mut ticks = interval(Duration::from_secs(60));
            ticks.set_missed_tick_behavior(MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = ticks.tick() => {
                        if let Err(error) = composition.cancellations.reconcile().await {
                            tracing::warn!(%error, "billing maintenance will retry");
                        }
                    },
                    _ = billing_stop.changed() => return Ok(()),
                }
            }
        }));
        let mut maintenance_stop = stop;
        let mut maintenance = OwnedTask(tokio::spawn(async move {
            let mut ticks = interval(Duration::from_secs(60));
            ticks.set_missed_tick_behavior(MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = ticks.tick() => {
                        tokio::select! {
                            result = composition.maintenance.reconcile() => {
                                if let Err(error) = result {
                                    tracing::warn!(%error, "dashboard maintenance will retry");
                                }
                            },
                            _ = maintenance_stop.changed() => return Ok(()),
                        }
                    },
                    _ = maintenance_stop.changed() => return Ok(()),
                }
            }
        }));
        let result = tokio::select! {
            result = &mut core.0 => result.internal()?,
            result = &mut http.0 => result.internal()?,
            result = &mut maintenance.0 => result.internal()?,
            result = &mut billing.0 => result.internal()?,
            _ = terminate.recv() => Ok(()),
            result = tokio::signal::ctrl_c() => result.internal(),
        };
        shutdown.send_replace(true);
        composition.hosted.ready.store(false, Ordering::Release);
        let _ = timeout(Duration::from_secs(240), async {
            if !core.0.is_finished() {
                let _ = (&mut core.0).await;
            }
            if !http.0.is_finished() {
                let _ = (&mut http.0).await;
            }
            if !maintenance.0.is_finished() {
                let _ = (&mut maintenance.0).await;
            }
            if !billing.0.is_finished() {
                let _ = (&mut billing.0).await;
            }
        })
        .await;
        result
    }
}
