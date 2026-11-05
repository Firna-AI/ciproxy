//! Download waits cannot block image holds. Runtime recovery removes old entries.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::{ImageDownload, ImageReport};
use futures_util::{pin_mut, poll};
use tokio::sync::{Mutex, oneshot};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::Images;
use crate::error::Result;
use crate::fixtures::metadata;
use crate::image_artifact::PendingImage;
use crate::image_download::{Download, ImageDownloadMock};
use crate::image_fixtures::{bundle, catalog, digest, image, pending};
use crate::image_store::ImageStoreMock;
use crate::image_verification::DownloadDigest;
use crate::images::ManagedImages;

struct WaitingDownload {
    delegate: Arc<dyn Download>,
    barrier: Mutex<Option<oneshot::Receiver<()>>>,
}

#[async_trait]
impl Download for WaitingDownload {
    async fn download(
        &self,
        image: &ImageDownload,
        pending: &PendingImage,
    ) -> Result<DownloadDigest> {
        let barrier = self.barrier.lock().await.take().unwrap();
        barrier.await.unwrap();
        self.delegate.download(image, pending).await
    }
}

#[tokio::test]
async fn the_current_image_remains_selectable_while_a_download_waits() {
    let mock = Arc::new(Unimock::new((
        ImageStoreMock::admit
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        ImageStoreMock::activate
            .some_call(matching!())
            .returns(Ok(())),
        ImageStoreMock::report
            .some_call(matching!())
            .returns(Ok(catalog())),
        ImageStoreMock::reclaim
            .each_call(matching!(_, _))
            .answers(&|_, _, _| Ok(())),
        ImageStoreMock::begin
            .some_call(matching!())
            .returns(Ok(pending())),
        ImageDownloadMock::download
            .some_call(matching!(_, _))
            .returns(Ok(digest(b"abc"))),
        ImageStoreMock::extract
            .some_call(matching!(_))
            .answers(&|_, checked| bundle(checked)),
        ImageStoreMock::publish
            .some_call(matching!(_))
            .returns(Ok(())),
    )));
    let (release, barrier) = oneshot::channel();
    let images = ManagedImages::new(
        mock.clone(),
        Arc::new(WaitingDownload {
            delegate: mock.clone(),
            barrier: Mutex::new(Some(barrier)),
        }),
    );
    images.recover().unwrap();
    let metadata = image();
    let fetch = images.fetch(&metadata);
    pin_mut!(fetch);
    assert!(poll!(fetch.as_mut()).is_pending());
    let selected = images.hold_newest().unwrap();
    assert_eq!(selected.metadata().image_id, ImageId("old".to_owned()));
    release.send(()).unwrap();
    fetch.await.unwrap();
    assert_eq!(
        images.report().unwrap().newest,
        Some(ImageId("new".to_owned()))
    );
}

#[test]
fn runtime_recovery_reclaims_unheld_images_and_abandoned_downloads() {
    let mock = Arc::new(Unimock::new((
        ImageStoreMock::admit.each_call(matching!(_)).answers(&|_, _| Ok(())),
        ImageStoreMock::activate.some_call(matching!()).returns(Ok(())),
        ImageStoreMock::report.some_call(matching!()).returns(Ok(ImageReport {
            available: vec![metadata("old", 10), metadata("obsolete", 15)], newest: Some(ImageId("old".to_owned())),
        })),
        ImageStoreMock::reclaim.some_call(matching!((keep, pending) if *keep == [ImageId("old".to_owned())] && pending.is_empty())).returns(Ok(())),
    )));
    let images = ManagedImages::new(mock.clone(), mock.clone());
    images.recover().unwrap();
    assert_eq!(images.report().unwrap(), catalog());
}
