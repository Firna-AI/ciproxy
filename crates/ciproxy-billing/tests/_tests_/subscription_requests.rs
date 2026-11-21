//! Literal request fixtures independent of production forms and response data.

use super::fake_subscriptions::{ChangeMode, ItemChange, SubscriptionRequest};

pub(super) fn business_upgrade() -> SubscriptionRequest {
    SubscriptionRequest::Change(
        ChangeMode::Upgrade,
        vec![
            ItemChange::Remove("si_tier"),
            ItemChange::Update("si_starts", "price_starts", 2),
            ItemChange::Update("si_machine", "price_machine", 1),
            ItemChange::Add("price_business", 1),
        ],
    )
}

pub(super) fn remove_starts() -> SubscriptionRequest {
    SubscriptionRequest::Change(
        ChangeMode::Prorated,
        vec![
            ItemChange::Update("si_tier", "price_team", 1),
            ItemChange::Remove("si_starts"),
            ItemChange::Update("si_machine", "price_machine", 1),
        ],
    )
}

pub(super) fn remove_remaining_machine() -> SubscriptionRequest {
    SubscriptionRequest::Change(
        ChangeMode::Prorated,
        vec![
            ItemChange::Update("si_tier", "price_team", 1),
            ItemChange::Remove("si_machine"),
        ],
    )
}

pub(super) fn remove_all_addons() -> SubscriptionRequest {
    SubscriptionRequest::Change(
        ChangeMode::Prorated,
        vec![
            ItemChange::Update("si_tier", "price_team", 1),
            ItemChange::Remove("si_starts"),
            ItemChange::Remove("si_machine"),
        ],
    )
}

pub(super) fn rejected_large_selection() -> SubscriptionRequest {
    SubscriptionRequest::Change(
        ChangeMode::Prorated,
        vec![
            ItemChange::Update("si_tier", "price_team", 1),
            ItemChange::Add("price_starts", 4_294_967_295),
            ItemChange::Add("price_machine", 1),
        ],
    )
}

pub(super) fn add_one_start_block() -> SubscriptionRequest {
    SubscriptionRequest::Change(
        ChangeMode::Prorated,
        vec![
            ItemChange::Update("si_tier", "price_team", 1),
            ItemChange::Add("price_starts", 1),
        ],
    )
}
