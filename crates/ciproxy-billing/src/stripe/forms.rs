//! Stable Stripe form construction for the monthly catalog.

use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
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
            if upgrade {
                "always_invoice"
            } else {
                "create_prorations"
            }
            .into(),
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
    let wanted = line_items(selection)?;
    let mut form = Vec::new();
    let mut index = 0;
    for old in &current.items.data {
        form.push((format!("{prefix}[{index}][id]"), old.id.clone()));
        if let Some(position) = wanted
            .iter()
            .position(|(code, _)| *code == old.price.lookup_key)
        {
            form.push((
                format!("{prefix}[{index}][price]"),
                prices[position].0.clone(),
            ));
            form.push((
                format!("{prefix}[{index}][quantity]"),
                prices[position].1.to_string(),
            ));
        } else {
            form.push((format!("{prefix}[{index}][deleted]"), "true".into()));
        }
        index += 1;
    }
    for (position, (code, _)) in wanted.iter().enumerate() {
        if !current
            .items
            .data
            .iter()
            .any(|item| item.price.lookup_key == *code)
        {
            form.push((
                format!("{prefix}[{index}][price]"),
                prices[position].0.clone(),
            ));
            form.push((
                format!("{prefix}[{index}][quantity]"),
                prices[position].1.to_string(),
            ));
            index += 1;
        }
    }
    Ok(form)
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
