//! Dated per-minute price catalogs with explicit incomplete comparisons.

use chrono::NaiveDate;
use ciproxy_store_interface::metering::totals::DailyTotal;
use ciproxy_store_interface::vm_size::VmSize;
use rust_decimal::Decimal;

/// An alternative to ciproxy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Alternative {
    /// Blacksmith-hosted GitHub Actions runners.
    Blacksmith,
    /// GitHub-hosted GitHub Actions runners.
    Github,
}

/// One list-priced hosted VM size in USD per minute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SizePrice {
    /// The vendor's virtual CPU count.
    pub vcpus: i32,
    /// The vendor's memory capacity in GB.
    pub memory_gb: i32,
    /// The list price per elapsed minute, with no invoice rounding.
    pub minute_usd: Decimal,
}

/// One explicitly supplied, dated vendor price list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceList {
    /// The vendor being compared.
    pub alternative: Alternative,
    /// The UTC date on which these list prices apply.
    pub dated: NaiveDate,
    /// The sizes and USD rates supplied by the caller.
    pub sizes: Vec<SizePrice>,
}

/// A resource match, without an invented ordering of incomparable sizes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PriceMatch {
    /// An exact resource match or uniquely smallest fitting size.
    Matched(SizePrice),
    /// The catalog sells no size with enough CPU and memory.
    Unavailable,
    /// Several smallest fitting sizes cannot be ordered by both resources.
    Ambiguous(Vec<SizePrice>),
}

/// An actual VM size whose comparison price remains unknown.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingPrice {
    /// The job VM resources that need a price.
    pub size: VmSize,
    /// Why the caller's catalog cannot select one price.
    pub price_match: PriceMatch,
}

/// The same jobs on one hosted alternative.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Comparison {
    /// The compared provider.
    pub alternative: Alternative,
    /// The date of the price list actually used.
    pub dated: NaiveDate,
    /// The total when every used size has a unique price.
    pub usd: Option<Decimal>,
    /// Missing or incomparable catalog entries, without hiding valid totals.
    pub missing: Vec<MissingPrice>,
}

/// Match both resources, preferring an exact match over a larger VM.
pub fn match_price(list: &PriceList, size: VmSize) -> PriceMatch {
    let fitting: Vec<_> = list
        .sizes
        .iter()
        .filter(|price| price.vcpus >= size.vcpus && price.memory_gb >= size.memory_gb)
        .collect();
    let smallest: Vec<_> = fitting
        .iter()
        .filter(|candidate| {
            !fitting.iter().any(|other| {
                other.vcpus <= candidate.vcpus
                    && other.memory_gb <= candidate.memory_gb
                    && (other.vcpus < candidate.vcpus || other.memory_gb < candidate.memory_gb)
            })
        })
        .map(|price| (*price).clone())
        .collect();
    match smallest.as_slice() {
        [] => PriceMatch::Unavailable,
        [price] => PriceMatch::Matched(price.clone()),
        _ => PriceMatch::Ambiguous(smallest),
    }
}

/// Price retained per-size daily totals without depending on retained job details.
pub fn compare(list: &PriceList, days: &[DailyTotal]) -> Comparison {
    let mut amount = Decimal::ZERO;
    let mut missing = Vec::new();
    for size in days.iter().flat_map(|day| &day.sizes) {
        let minutes = size.machines.minutes + size.boat.minutes;
        if minutes == Decimal::ZERO {
            continue;
        }
        match match_price(list, size.size) {
            PriceMatch::Matched(price) => amount += price.minute_usd * minutes,
            price_match => {
                let issue = MissingPrice {
                    size: size.size,
                    price_match,
                };
                if !missing.contains(&issue) {
                    missing.push(issue);
                }
            }
        }
    }
    Comparison {
        alternative: list.alternative,
        dated: list.dated,
        usd: missing.is_empty().then_some(amount),
        missing,
    }
}

#[cfg(test)]
#[path = "../_tests_/catalog_tests.rs"]
mod catalog_tests;
