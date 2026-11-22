//! A resumable image download against a local HTTP host: resume inside one
//! command and across commands, restarts, errors that keep the bytes, the
//! final check, and recovery.

use std::path::{Path, PathBuf};
use std::time::Duration;

use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::secret::Secret;
use sha2::{Digest, Sha256};

use super::{ResumableDownload, Retry, recover};
use crate::error::Error;
use crate::range_server::{RangeServer, Reply};

/// A case name, the wanted body, and a change to the partial file.
type Case = (&'static str, &'static [u8], fn(&Path));

const BODY: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
const OTHER_BODY: &[u8] = b"ZYXWVUTSRQPONMLKJIHGFEDCBA9876543210";

/// A new images folder under the system temporary folder, removed on drop.
struct Folder(PathBuf);

impl Folder {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("ciproxy-resume-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn archive(&self, name: &str) -> PathBuf {
        self.0.join(format!("{name}.tar"))
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The partial file and the sidecar of `archive`, in the download's names.
fn partial_files(archive: &Path) -> [PathBuf; 2] {
    let name = archive.display();
    [format!("{name}.partial"), format!("{name}.partial.json")].map(PathBuf::from)
}

fn image(id: &str, body: &[u8], server: &RangeServer) -> ImageDownload {
    ImageDownload {
        image_id: ImageId(id.to_owned()),
        platform: Platform::MacosArm64,
        download_url: Secret::new(server.url.clone()),
        sha256: hex::encode(Sha256::digest(body)),
        size_bytes: body.len() as u64,
        root_disk_bytes: 0,
    }
}

fn download(retries: u32) -> ResumableDownload {
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    ResumableDownload::new(
        client,
        Retry {
            retries,
            wait: Duration::ZERO,
        },
    )
}

/// Leave the first five bytes of `body` as the partial file of `archive`.
async fn break_after_five(id: &str, body: &[u8], archive: &Path) {
    let server = RangeServer::start(body, vec![Reply::Break(5)]).await;
    let result = download(0).fetch(&image(id, body, &server), archive).await;
    assert!(matches!(result, Err(Error::Internal(_))), "{result:?}");
}

fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap()
}

#[tokio::test]
async fn fresh_download_writes_the_checked_archive_and_removes_other_partials() {
    let folder = Folder::new();
    let (archive, other) = (folder.archive("macos-2"), folder.archive("macos-1"));
    break_after_five("macos-1", OTHER_BODY, &other).await;
    let server = RangeServer::start(BODY, vec![Reply::Serve]).await;
    download(0)
        .fetch(&image("macos-2", BODY, &server), &archive)
        .await
        .unwrap();
    assert_eq!(read(&archive), BODY);
    assert_eq!(server.ranges(), vec![None]);
    for file in partial_files(&archive).iter().chain(&partial_files(&other)) {
        assert!(!file.exists(), "{}", file.display());
    }
}

#[tokio::test]
async fn broken_stream_continues_with_range_in_the_same_command() {
    let folder = Folder::new();
    let archive = folder.archive("macos-1");
    let server = RangeServer::start(BODY, vec![Reply::Break(5), Reply::Serve]).await;
    download(1)
        .fetch(&image("macos-1", BODY, &server), &archive)
        .await
        .unwrap();
    assert_eq!(read(&archive), BODY);
    assert_eq!(server.ranges(), vec![None, Some("bytes=5-".to_owned())]);
}

#[tokio::test]
async fn next_command_continues_the_partial_file_of_its_sidecar() {
    let folder = Folder::new();
    let archive = folder.archive("macos-1");
    break_after_five("macos-1", BODY, &archive).await;
    let server = RangeServer::start(BODY, vec![Reply::Serve]).await;
    download(0)
        .fetch(&image("macos-1", BODY, &server), &archive)
        .await
        .unwrap();
    assert_eq!(read(&archive), BODY);
    assert_eq!(server.ranges(), vec![Some("bytes=5-".to_owned())]);
}

#[tokio::test]
async fn partial_file_starts_again_when_its_sidecar_or_length_does_not_match() {
    let longer = |archive: &Path| {
        let [partial, _] = partial_files(archive);
        std::fs::write(partial, [BODY, b"!"].concat()).unwrap();
    };
    let cases: [Case; 2] = [
        ("another SHA-256", OTHER_BODY, |_| {}),
        ("longer than the image", BODY, longer),
    ];
    for (case, wanted, change) in cases {
        let folder = Folder::new();
        let archive = folder.archive("macos-1");
        break_after_five("macos-1", BODY, &archive).await;
        change(&archive);
        let server = RangeServer::start(wanted, vec![Reply::Serve]).await;
        download(0)
            .fetch(&image("macos-1", wanted, &server), &archive)
            .await
            .unwrap();
        assert_eq!(read(&archive), wanted, "{case}");
        assert_eq!(server.ranges(), vec![None], "{case}");
    }
}

#[tokio::test]
async fn whole_answer_to_a_range_request_starts_the_file_again() {
    let folder = Folder::new();
    let archive = folder.archive("macos-1");
    let server = RangeServer::start(BODY, vec![Reply::Break(5), Reply::IgnoreRange]).await;
    download(1)
        .fetch(&image("macos-1", BODY, &server), &archive)
        .await
        .unwrap();
    assert_eq!(read(&archive), BODY);
    assert_eq!(server.ranges(), vec![None, Some("bytes=5-".to_owned())]);
}

#[tokio::test]
async fn refused_answer_ends_the_command_and_keeps_the_partial_file() {
    let cases = [
        (Reply::Status(403), Error::ImageStatus { status: 403 }),
        (Reply::Status(503), Error::ImageStatus { status: 503 }),
    ];
    for (reply, expected) in cases {
        let folder = Folder::new();
        let archive = folder.archive("macos-1");
        let server = RangeServer::start(BODY, vec![Reply::Break(5), reply, Reply::Serve]).await;
        let result = download(3)
            .fetch(&image("macos-1", BODY, &server), &archive)
            .await;
        assert_eq!(result.unwrap_err().to_string(), expected.to_string());
        assert_eq!(server.ranges().len(), 2, "{reply:?}");
        let [partial, sidecar] = partial_files(&archive);
        assert_eq!(read(&partial), &BODY[..5], "{reply:?}");
        assert!(sidecar.is_file(), "{reply:?}");
        assert!(!archive.exists(), "{reply:?}");
    }
}

#[tokio::test]
async fn range_answer_that_cannot_continue_starts_again_without_range() {
    for reply in [Reply::Shifted, Reply::Status(416)] {
        let folder = Folder::new();
        let archive = folder.archive("macos-1");
        let server = RangeServer::start(BODY, vec![Reply::Break(5), reply, Reply::Serve]).await;
        download(3)
            .fetch(&image("macos-1", BODY, &server), &archive)
            .await
            .unwrap();
        assert_eq!(read(&archive), BODY, "{reply:?}");
        let expected = vec![None, Some("bytes=5-".to_owned()), None];
        assert_eq!(server.ranges(), expected, "{reply:?}");
    }
}

#[tokio::test]
async fn final_mismatch_deletes_the_partial_file_and_its_sidecar() {
    let folder = Folder::new();
    let archive = folder.archive("macos-1");
    let server = RangeServer::start(BODY, vec![Reply::Serve]).await;
    let mut wrong = image("macos-1", BODY, &server);
    wrong.sha256 = hex::encode(Sha256::digest(OTHER_BODY));
    let result = download(0).fetch(&wrong, &archive).await;
    assert!(
        matches!(result, Err(Error::ImageVerification)),
        "{result:?}"
    );
    for file in partial_files(&archive).iter().chain([&archive]) {
        assert!(!file.exists(), "{}", file.display());
    }
}

#[tokio::test]
async fn recovery_keeps_a_partial_file_with_a_valid_sidecar_only() {
    let folder = Folder::new();
    let kept = folder.archive("macos-1");
    break_after_five("macos-1", BODY, &kept).await;
    let [orphan, _] = partial_files(&folder.archive("macos-2"));
    std::fs::write(&orphan, b"0123").unwrap();
    let [_, broken] = partial_files(&folder.archive("macos-3"));
    std::fs::write(&broken, b"not json").unwrap();

    recover(&folder.0).unwrap();

    let [partial, sidecar] = partial_files(&kept);
    assert_eq!(read(&partial), &BODY[..5]);
    assert!(sidecar.is_file());
    assert!(!orphan.exists());
    assert!(!broken.exists());
}
