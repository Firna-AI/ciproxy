//! Trusted Linux x64 USD list prices, checked against the vendors on their date.

use chrono::NaiveDate;
use ciproxy_metering::report::catalog::{Alternative, PriceList, SizePrice};
use rust_decimal::Decimal;

use crate::error::{ErrorResultExt, Result};

pub(crate) fn configured() -> Result<[PriceList; 2]> {
    let dated = NaiveDate::parse_from_str("2026-11-07", "%Y-%m-%d").internal()?;
    Ok([
        PriceList {
            alternative: Alternative::Blacksmith,
            dated,
            sizes: sizes(&[
                (2, 8, 4),
                (4, 16, 8),
                (8, 32, 16),
                (16, 64, 32),
                (32, 128, 64),
            ]),
        },
        PriceList {
            alternative: Alternative::Github,
            dated,
            sizes: sizes(&[
                (2, 8, 6),
                (4, 16, 12),
                (8, 32, 22),
                (16, 64, 42),
                (32, 128, 82),
                (64, 256, 162),
                (96, 384, 252),
            ]),
        },
    ])
}

fn sizes(values: &[(i32, i32, i64)]) -> Vec<SizePrice> {
    values
        .iter()
        .map(|&(vcpus, memory_gb, mills)| SizePrice {
            vcpus,
            memory_gb,
            minute_usd: Decimal::new(mills, 3),
        })
        .collect()
}
