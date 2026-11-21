//! Exact shared-capability catalog mapping without invented dimensions or prices.

use ciproxy_protocol::resources::VmSize;
use ciproxy_scaler::boat::{BoatSize, BoatSizeId};
use ciproxy_store_interface::overflow::account::{AccountState, SizeOffer};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use sandbox_interface::{ResourceAmount, ResourceUnit, SandboxPriceUnit, SandboxSizeOffer};

use crate::error::{Error, Result};

/// Known physical sizes, independent of spend and temporary start refusals.
pub struct ResourceCatalog {
    /// Account sizes with their trusted dimensions and selection prices.
    pub sizes: Vec<BoatSize>,
    /// The largest size from the provider's ordered account catalog.
    pub largest_size: Option<BoatSizeId>,
}

pub(crate) fn resources(state: &AccountState) -> Result<ResourceCatalog> {
    let sizes = state
        .sizes
        .iter()
        .map(resource_size)
        .collect::<Result<Vec<_>>>()?;
    Ok(ResourceCatalog {
        largest_size: sizes.last().map(|size| size.id.clone()),
        sizes,
    })
}

/// Map eligible catalog facts without reading or requiring any cost facts.
pub(crate) fn scaler_sizes(state: &AccountState) -> Result<Vec<BoatSize>> {
    state
        .sizes
        .iter()
        .filter(|size| !state.refused_sizes.contains(&size.name))
        .map(resource_size)
        .collect()
}

fn resource_size(size: &SizeOffer) -> Result<BoatSize> {
    Ok(BoatSize {
        id: BoatSizeId(size.name.clone()),
        size: VmSize {
            vcpu: size.vcpus,
            memory_bytes: size.memory_bytes,
            disk_bytes: size.disk_bytes,
        },
        price: micros(size.hourly_usd)?,
    })
}

pub(crate) fn micros(value: Decimal) -> Result<u64> {
    (value * Decimal::from(1_000_000))
        .ceil()
        .to_u64()
        .ok_or(Error::InvalidProviderData)
}

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
