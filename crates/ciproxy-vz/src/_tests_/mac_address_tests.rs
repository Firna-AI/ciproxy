//! Text forms and random addresses.

use super::MacAddress;

#[test]
fn display_pads_each_octet_and_lease_form_drops_leading_zeros() {
    let mac = MacAddress::new([0xa2, 0x4c, 0x08, 0x01, 0x00, 0x0f]);
    assert_eq!(mac.to_string(), "a2:4c:08:01:00:0f");
    assert_eq!(mac.lease_form(), "a2:4c:8:1:0:f");
}

#[test]
fn parse_reads_the_normal_and_the_lease_form() {
    let mac = MacAddress::new([0xa2, 0x4c, 0x08, 0x01, 0x00, 0x0f]);
    assert_eq!("a2:4c:08:01:00:0f".parse::<MacAddress>().unwrap(), mac);
    assert_eq!("A2:4C:8:1:0:F".parse::<MacAddress>().unwrap(), mac);
}

#[test]
fn parse_rejects_wrong_shapes() {
    for text in [
        "",
        "a2:4c:08:01:00",
        "a2:4c:08:01:00:0f:00",
        "a2:4c:08:01::0f",
        "a2:4c:08:01:000:0f",
        "a2:4c:08:01:+f:0f",
        "a2:4c:08:01:0g:0f",
    ] {
        assert!(text.parse::<MacAddress>().is_err(), "{text:?}");
    }
}

#[test]
fn random_addresses_are_local_unicast() {
    let mac = MacAddress::from_random([0xff; 6]);
    assert_eq!(mac.octets()[0], 0xfe);
    let mac = MacAddress::from_random([0x00; 6]);
    assert_eq!(mac.octets()[0], 0x02);
    let mac = MacAddress::random_local().unwrap();
    assert_eq!(mac.octets()[0] & 0b11, 0b10);
}
