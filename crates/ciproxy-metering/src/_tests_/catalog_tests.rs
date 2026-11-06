//! Exact and smallest-larger resource matching without a guessed catalog.

use crate::report::catalog::{Alternative, PriceMatch, SizePrice, match_price};
use crate::test_support::{catalog, size, usd};

#[test]
fn catalog_match_requires_both_cpu_and_memory() {
    let mut list = catalog(Alternative::Blacksmith, "0.004");
    list.sizes.extend([
        SizePrice {
            vcpus: 4,
            memory_gb: 8,
            minute_usd: usd("0.007"),
        },
        SizePrice {
            vcpus: 4,
            memory_gb: 16,
            minute_usd: usd("0.008"),
        },
        SizePrice {
            vcpus: 8,
            memory_gb: 32,
            minute_usd: usd("0.016"),
        },
    ]);
    assert_eq!(
        match_price(&list, size()),
        PriceMatch::Matched(SizePrice {
            vcpus: 2,
            memory_gb: 8,
            minute_usd: usd("0.004")
        })
    );
    assert_eq!(
        match_price(
            &list,
            ciproxy_protocol::resources::VmSize {
                vcpu: 3,
                memory_bytes: 12_000_000_000,
                ..size()
            }
        ),
        PriceMatch::Matched(SizePrice {
            vcpus: 4,
            memory_gb: 16,
            minute_usd: usd("0.008")
        })
    );
    assert_eq!(
        match_price(
            &list,
            ciproxy_protocol::resources::VmSize { vcpu: 16, ..size() }
        ),
        PriceMatch::Unavailable
    );
}

#[test]
fn incomparable_fitting_sizes_remain_explicitly_ambiguous() {
    let mut list = catalog(Alternative::Github, "0.006");
    let candidates = vec![
        SizePrice {
            vcpus: 4,
            memory_gb: 32,
            minute_usd: usd("0.02"),
        },
        SizePrice {
            vcpus: 8,
            memory_gb: 16,
            minute_usd: usd("0.03"),
        },
    ];
    list.sizes = candidates.clone();
    assert_eq!(
        match_price(
            &list,
            ciproxy_protocol::resources::VmSize {
                vcpu: 4,
                memory_bytes: 16_000_000_000,
                ..size()
            }
        ),
        PriceMatch::Ambiguous(candidates)
    );
}
