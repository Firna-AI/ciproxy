//! The image checks refuse anything that the released format does not name.

use std::io::{Cursor, Write};

use ciproxy_protocol::identifiers::ImageId;

use super::image_bundle::{
    ImageIndex, IndexedImage, MEMBERS, Manifest, SPARSE_BLOCK, SparseWriter, folder_name,
    parse_sums, summed_files,
};

fn sums() -> String {
    summed_files()
        .enumerate()
        .map(|(index, file)| format!("{:064x}  {file}\n", index + 1))
        .collect()
}

#[test]
fn the_sum_file_names_each_file_once() {
    let parsed = parse_sums(&sums()).expect("sums");
    assert_eq!(parsed.len(), MEMBERS.len());
    assert!(parsed.contains_key("bundle/disk.img"));
    assert!(!parsed.contains_key("macos-content.sha256"));
    let starred = sums().replacen("  bundle/aux.img", " *bundle/aux.img", 1);
    assert!(parse_sums(&starred).is_some());
}

#[test]
fn the_sum_file_refuses_gaps_repeats_and_bad_digests() {
    let missing: String = sums()
        .lines()
        .filter(|line| !line.ends_with("bundle/disk.img"))
        .map(|line| format!("{line}\n"))
        .collect();
    let repeated = format!("{}{:064x}  manifest.json\n", sums(), 9);
    let bad = sums().replacen(&format!("{:064x}", 1), &format!("{:z<64}", ""), 1);
    let short = format!("{}abc  extra\n", sums());
    for text in [missing, repeated, bad, short] {
        assert_eq!(parse_sums(&text), None, "{text}");
    }
}

#[test]
fn the_manifest_gives_the_minimum_host() {
    let manifest: Manifest = serde_json::from_str(
        r#"{"minimum_host_macos":"26.7.3","free_job_bytes":107374182400,"build":"25G342"}"#,
    )
    .expect("manifest");
    assert_eq!(manifest.minimum_host_macos, "26.7.3");
    assert_eq!(manifest.free_job_bytes, 100 << 30);
}

fn image(id: &str) -> IndexedImage {
    IndexedImage {
        image_id: ImageId(id.to_owned()),
        root_disk_bytes: 200 << 30,
        minimum_host_macos: "26.7.3".to_owned(),
        free_job_bytes: 100 << 30,
    }
}

#[test]
fn the_index_keeps_the_newest_and_held_images() {
    let mut index = ImageIndex::default();
    index.publish(image("a"));
    index.publish(image("b"));
    index.publish(image("c"));
    assert_eq!(
        index.newest().map(|image| image.image_id.0.as_str()),
        Some("c")
    );
    let removed = index.reclaim(&[ImageId("a".to_owned())]);
    assert_eq!(removed, [ImageId("b".to_owned())]);
    let report = index.report();
    assert_eq!(report.available.len(), 2);
    assert_eq!(report.newest, Some(ImageId("c".to_owned())));
    index.publish(image("a"));
    assert_eq!(index.images.len(), 2);
    assert_eq!(index.newest, Some(ImageId("a".to_owned())));
}

/// The report carries each image's measured free bytes on the wire, so the
/// control plane can apply the image limit of a macOS label.
#[test]
fn the_report_sends_the_measured_free_bytes_of_each_image() {
    let mut index = ImageIndex::default();
    index.publish(image("a"));

    let report = serde_json::to_value(index.report()).expect("encode the report");

    assert_eq!(
        report["available"][0]["free_job_bytes"],
        serde_json::json!(100_u64 << 30)
    );
    assert_eq!(index.images[0].metadata().free_job_bytes, Some(100 << 30));
}

#[test]
fn folder_names_never_follow_the_id() {
    let name = folder_name(&ImageId("../../etc".to_owned()));
    assert_eq!(name.len(), 64);
    assert!(name.bytes().all(|byte| byte.is_ascii_hexdigit()));
}

#[test]
fn zero_blocks_become_holes() {
    let mut data = vec![0_u8; SPARSE_BLOCK * 3 + 10];
    data[SPARSE_BLOCK + 5] = 7;
    let mut writer = SparseWriter::new(Cursor::new(Vec::new()));
    writer.write_all(&data).expect("write");
    let (cursor, length) = writer.finish().expect("finish");
    assert_eq!(length, data.len() as u64);
    let written = cursor.into_inner();
    assert_eq!(
        written.len(),
        SPARSE_BLOCK * 2,
        "only the data block is written"
    );
    assert_eq!(written[SPARSE_BLOCK + 5], 7);
}
