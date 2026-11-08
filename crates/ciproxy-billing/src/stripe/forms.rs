//! Stable Stripe form construction for the monthly catalog.

use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::quotes::{ChangeItem, QuotePayment};
use ciproxy_store_interface::billing::state::Selection;
use uuid::Uuid;

use crate::catalog::line_items;
use crate::error::{Error, Result};
use crate::stripe::wire::types::Subscription;

pub(super) fn checkout(workspace: Uuid, command: &BillingCommand) -> Result<Vec<(String, String)>> {
    let CommandKind::Checkout {
        selection: _,
        success_url,
        cancel_url,
    } = &command.kind
    else {
        return Err(Error::StripeState);
    };
    Ok(vec![
        ("mode".into(), "subscription".into()),
        ("allowed_payment_method_types[0]".into(), "card".into()),
        ("automatic_tax[enabled]".into(), "false".into()),
        ("tax_id_collection[enabled]".into(), "false".into()),
        ("success_url".into(), success_url.clone()),
        ("cancel_url".into(), cancel_url.clone()),
        ("metadata[workspace_id]".into(), workspace.to_string()),
        ("metadata[command_id]".into(), command.id.to_string()),
        (
            "subscription_data[metadata][workspace_id]".into(),
            workspace.to_string(),
        ),
        (
            "subscription_data[metadata][command_id]".into(),
            command.id.to_string(),
        ),
    ])
}

pub(super) fn replacement(
    current: &Subscription,
    selection: Selection,
    prices: &[(String, u32)],
    upgrade: bool,
) -> Result<Vec<(String, String)>> {
    let mut form = vec![
        (
            "proration_behavior".into(),
            proration_behavior(upgrade).into(),
        ),
        (
            "payment_behavior".into(),
            if upgrade {
                "pending_if_incomplete"
            } else {
                "allow_incomplete"
            }
            .into(),
        ),
    ];
    form.extend(items(current, selection, prices, "items")?);
    Ok(form)
}

pub(super) fn items(
    current: &Subscription,
    selection: Selection,
    prices: &[(String, u32)],
    prefix: &str,
) -> Result<Vec<(String, String)>> {
    Ok(item_fields(&changes(current, selection, prices)?, prefix))
}

pub(super) fn changes(
    current: &Subscription,
    selection: Selection,
    prices: &[(String, u32)],
) -> Result<Vec<ChangeItem>> {
    let wanted = line_items(selection)?;
    let mut items = Vec::new();
    for old in &current.items.data {
        if let Some(position) = wanted
            .iter()
            .position(|(code, _)| *code == old.price.lookup_key)
        {
            items.push(ChangeItem::Set {
                id: Some(old.id.clone()),
                price_id: prices[position].0.clone(),
                quantity: prices[position].1,
            });
        } else {
            items.push(ChangeItem::Remove { id: old.id.clone() });
        }
    }
    for (position, (code, _)) in wanted.iter().enumerate() {
        if !current
            .items
            .data
            .iter()
            .any(|item| item.price.lookup_key == *code)
        {
            items.push(ChangeItem::Set {
                id: None,
                price_id: prices[position].0.clone(),
                quantity: prices[position].1,
            });
        }
    }
    Ok(items)
}

pub(super) fn item_fields(items: &[ChangeItem], prefix: &str) -> Vec<(String, String)> {
    let mut form = Vec::new();
    for (index, item) in items.iter().enumerate() {
        match item {
            ChangeItem::Remove { id } => {
                form.push((format!("{prefix}[{index}][id]"), id.clone()));
                form.push((format!("{prefix}[{index}][deleted]"), "true".into()));
            }
            ChangeItem::Set {
                id,
                price_id,
                quantity,
            } => {
                if let Some(id) = id {
                    form.push((format!("{prefix}[{index}][id]"), id.clone()));
                }
                form.push((format!("{prefix}[{index}][price]"), price_id.clone()));
                form.push((format!("{prefix}[{index}][quantity]"), quantity.to_string()));
            }
        }
    }
    form
}

pub(super) fn reviewed(
    quote: &QuotePayment,
    upgrade: bool,
    preview: bool,
) -> Vec<(String, String)> {
    let name = |field: &str| {
        if preview {
            format!("subscription_details[{field}]")
        } else {
            field.into()
        }
    };
    let mut form = vec![
        (
            name("proration_behavior"),
            proration_behavior(upgrade).into(),
        ),
        (
            name("proration_date"),
            quote.proration_at.timestamp().to_string(),
        ),
    ];
    if !preview {
        form.push((
            "payment_behavior".into(),
            if upgrade {
                "pending_if_incomplete"
            } else {
                "allow_incomplete"
            }
            .into(),
        ));
    }
    form.extend(item_fields(&quote.items, &name("items")));
    form
}

fn proration_behavior(upgrade: bool) -> &'static str {
    if upgrade {
        "always_invoice"
    } else {
        "create_prorations"
    }
}

pub(super) fn portal_configuration() -> Vec<(String, String)> {
    [
        ("features[invoice_history][enabled]", "true"),
        ("features[payment_method_update][enabled]", "true"),
        ("features[subscription_cancel][enabled]", "true"),
        ("features[subscription_cancel][mode]", "at_period_end"),
        ("features[subscription_cancel][proration_behavior]", "none"),
        ("features[subscription_update][enabled]", "false"),
        ("features[customer_update][enabled]", "false"),
    ]
    .into_iter()
    .map(|(key, value)| (key.into(), value.into()))
    .collect()
}
