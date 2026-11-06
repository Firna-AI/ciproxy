//! Images::fetch cannot publish failed checks and retains active preparation bases.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageReport;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::Images;
use crate::error::Error;
use crate::fixtures::metadata;
use crate::image_download::ImageDownloadMock;
use crate::image_fixtures::{bundle, catalog, digest, image, pending};
use crate::image_store::ImageStoreMock;
use crate::images::ManagedImages;

#[tokio::test]
async fn fetch_rejects_corrupt_or_truncated_bytes_and_preserves_the_old_newest() {
    for bytes in [b"abd".as_slice(), b"ab", b"abcd"] {
        let actual = digest(bytes);
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
                .each_call(matching!((keep, _) if *keep == [ImageId("old".to_owned())]))
                .answers(&|_, _, _| Ok(())),
            ImageStoreMock::begin
                .some_call(matching!())
                .returns(Ok(pending())),
            ImageDownloadMock::download
                .some_call(matching!(_, _))
                .returns(Ok(actual)),
        )));
        let images = ManagedImages::new(mock.clone(), mock.clone());
        images.recover().unwrap();
        assert!(
            matches!(images.fetch(&image()).await, Err(Error::ImageVerification)),
            "{bytes:?}"
        );
        assert_eq!(images.report().unwrap(), catalog());
    }
}

#[tokio::test]
async fn malformed_bundle_never_replaces_the_old_newest() {
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
            .returns(Err(Error::ImageLayout)),
    )));
    let images = ManagedImages::new(mock.clone(), mock.clone());
    images.recover().unwrap();
    assert!(matches!(
        images.fetch(&image()).await,
        Err(Error::ImageLayout)
    ));
    assert_eq!(images.report().unwrap(), catalog());
}

#[tokio::test]
async fn publication_selects_the_newest_and_reclaims_the_old_base_after_its_hold() {
    let kept = Arc::new(Mutex::new(Vec::new()));
    let observations = kept.clone();
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
            .answers_arc(Arc::new(move |_, keep, pending| {
                observations
                    .lock()
                    .unwrap()
                    .push((keep.to_vec(), pending.len()));
                Ok(())
            })),
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
            .some_call(matching!((bundle) if bundle.metadata().image_id.0 == "new"))
            .returns(Ok(())),
    )));
    let images = ManagedImages::new(mock.clone(), mock.clone());
    images.recover().unwrap();
    {
        let old = images.hold_newest().unwrap();
        assert_eq!(old.metadata(), metadata("old", 10));
        images.fetch(&image()).await.unwrap();
        assert_eq!(
            images.report().unwrap().newest,
            Some(ImageId("new".to_owned()))
        );
        let retained = kept.lock().unwrap().last().unwrap().0.clone();
        assert_eq!(
            retained,
            vec![ImageId("new".to_owned()), ImageId("old".to_owned())]
        );
        let new = images.hold_newest().unwrap();
        assert_eq!(new.metadata(), metadata("new", 20));
        assert_eq!(old.metadata(), metadata("old", 10));
        let saved = serde_json::to_vec(&images.report().unwrap()).unwrap();
        let restored: ImageReport = serde_json::from_slice(&saved).unwrap();
        assert_eq!(restored.selected(), Some(&metadata("new", 20)));
    }
    assert_eq!(
        images.report().unwrap().available,
        vec![metadata("new", 20)]
    );
    assert_eq!(
        kept.lock().unwrap().last().unwrap().0,
        vec![ImageId("new".to_owned())]
    );
    assert_eq!(kept.lock().unwrap().last().unwrap().1, 0);
    images.purge().unwrap();
    assert_eq!(images.report().unwrap().newest, None);
    assert_eq!(
        kept.lock().unwrap().last().unwrap().0,
        Vec::<ImageId>::new()
    );
}

#[test]
fn reading_an_install_report_never_activates_or_reclaims_an_active_folder() {
    let mock = Arc::new(Unimock::new(
        ImageStoreMock::report
            .some_call(matching!())
            .returns(Ok(catalog())),
    ));
    let images = ManagedImages::new(mock.clone(), mock.clone());
    assert_eq!(images.report().unwrap(), catalog());
}
