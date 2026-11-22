//! Lease file parsing and lookup.

use std::net::Ipv4Addr;

use super::{Lease, find, lookup, parse};
use crate::mac_address::MacAddress;

const LEASES: &str = "{
\tname=admin-vm
\tip_address=192.168.64.3
\thw_address=1,a2:4c:8:1:0:f
\tidentifier=1,a2:4c:8:1:0:f
\tlease=0x6740a1b2
}
{
\tname=admin-vm
\tip_address=192.168.64.7
\thw_address=1,a2:4c:8:1:0:f
\tlease=0x6740a1b3
}
{
\tip_address=192.168.64.4
\thw_address=1,de:ad:be:ef:0:1
}
{
\tname=broken
\thw_address=1,de:ad:be:ef:0:2
}
";

fn mac() -> MacAddress {
    MacAddress::new([0xa2, 0x4c, 0x08, 0x01, 0x00, 0x0f])
}

#[test]
fn parse_reads_each_complete_block_and_skips_incomplete_ones() {
    let leases = parse(LEASES);
    assert_eq!(leases.len(), 3);
    assert_eq!(
        leases[0],
        Lease {
            name: Some("admin-vm".to_owned()),
            ip_address: Ipv4Addr::new(192, 168, 64, 3),
            hardware: mac(),
            expires: 0x6740_a1b2,
        }
    );
    assert_eq!(leases[2].name, None);
    assert_eq!(leases[2].expires, 0);
}

#[test]
fn find_takes_the_newest_lease_of_the_address() {
    assert_eq!(find(LEASES, mac()), Some(Ipv4Addr::new(192, 168, 64, 7)));
    let other = MacAddress::new([0xde, 0xad, 0xbe, 0xef, 0x00, 0x01]);
    assert_eq!(find(LEASES, other), Some(Ipv4Addr::new(192, 168, 64, 4)));
    let missing = MacAddress::new([0xde, 0xad, 0xbe, 0xef, 0x00, 0x03]);
    assert_eq!(find(LEASES, missing), None);
}

#[test]
fn parse_ignores_an_unclosed_block_and_text_outside_blocks() {
    let text = "ip_address=10.0.0.1\n{\n\tip_address=10.0.0.2\n\thw_address=1,a2:4c:8:1:0:f\n";
    assert!(parse(text).is_empty());
}

#[test]
fn lookup_reads_the_file_and_treats_a_missing_file_as_no_lease() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("dhcpd_leases");
    assert_eq!(lookup(&path, mac()).unwrap(), None);
    std::fs::write(&path, LEASES).unwrap();
    assert_eq!(
        lookup(&path, mac()).unwrap(),
        Some(Ipv4Addr::new(192, 168, 64, 7))
    );
}
