//! Exact shared-capability catalog mapping without invented dimensions or prices.

use ciproxy_store_interface::overflow::account::SizeOffer;
use rust_decimal::Decimal;
use sandbox_interface::{ResourceAmount, ResourceUnit, SandboxPriceUnit, SandboxSizeOffer};

use crate::error::{Error, Result};

pub(crate) fn offers(offers: Vec<SandboxSizeOffer>) -> Result<Vec<SizeOffer>> {
    offers
        .into_iter()
        .map(|offer| {
            if offer.hourly_price.currency != "USD"
                || offer.hourly_price.unit != SandboxPriceUnit::Hour
                || offer.hourly_price.amount.scale > 28
            {
                return Err(Error::InvalidProviderData);
            }
            Ok(SizeOffer {
                name: offer.provider_size,
                vcpus: offer.resources.vcpus,
                memory_bytes: bytes(offer.resources.memory)?,
                disk_bytes: bytes(offer.resources.disk)?,
                hourly_usd: Decimal::from_i128_with_scale(
                    i128::from(offer.hourly_price.amount.coefficient),
                    u32::from(offer.hourly_price.amount.scale),
                ),
            })
        })
        .collect()
}

fn bytes(amount: ResourceAmount) -> Result<u64> {
    let factor = match amount.unit {
        ResourceUnit::Bytes => 1,
        ResourceUnit::Megabytes => 1_000_000,
        ResourceUnit::Gigabytes => 1_000_000_000,
        ResourceUnit::Mebibytes => 1_048_576,
        ResourceUnit::Gibibytes => 1_073_741_824,
    };
    amount
        .amount
        .checked_mul(factor)
        .ok_or(Error::InvalidProviderData)
}
