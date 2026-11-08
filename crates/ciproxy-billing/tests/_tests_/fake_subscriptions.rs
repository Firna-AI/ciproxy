//! Mandatory literal subscription request and outcome pairs.

use ciproxy_billing::stripe::wire::types::Subscription;

use super::fake_requests::Request;

#[derive(Clone, Debug)]
pub(super) enum ItemChange {
    Update(&'static str, &'static str, u32),
    Add(&'static str, u32),
    Remove(&'static str),
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ChangeMode {
    Upgrade,
    Prorated,
}

#[derive(Clone, Debug)]
pub(super) enum SubscriptionRequest {
    Reviewed(Box<SubscriptionRequest>, i64),
    Change(ChangeMode, Vec<ItemChange>),
    CancelAtPeriodEnd,
    CancelImmediately(String),
}

pub(super) enum SubscriptionOutcome {
    Respond(Box<Subscription>),
    Reject(u16),
}

impl SubscriptionOutcome {
    pub(super) fn respond(subscription: Subscription) -> Self {
        Self::Respond(Box::new(subscription))
    }
}

pub(super) struct SubscriptionExchange {
    expected: SubscriptionRequest,
    outcome: SubscriptionOutcome,
}

impl SubscriptionExchange {
    pub(super) fn new(expected: SubscriptionRequest, outcome: SubscriptionOutcome) -> Self {
        Self { expected, outcome }
    }

    pub(super) fn describe(&self) -> String {
        format!("{:?}", self.expected)
    }

    pub(super) fn consume(self, request: &Request) -> (Vec<String>, SubscriptionOutcome) {
        let mut expected = self.expected.fields();
        let mut actual = request.form.clone();
        expected.sort();
        actual.sort();
        let (method, path) = match &self.expected {
            SubscriptionRequest::CancelImmediately(id) => {
                ("DELETE", format!("/v1/subscriptions/{id}"))
            }
            _ => ("POST", "/v1/subscriptions/sub_recorded".into()),
        };
        let errors = if expected == actual && request.method == method && request.path == path {
            Vec::new()
        } else {
            let differences = expected
                .iter()
                .filter(|field| !actual.contains(field))
                .map(|(key, value)| {
                    let received = actual
                        .iter()
                        .filter(|(name, _)| name == key)
                        .map(|(_, value)| value.as_str())
                        .collect::<Vec<_>>();
                    format!("{key}: expected {value:?}, actual {received:?}")
                })
                .chain(
                    actual
                        .iter()
                        .filter(|field| !expected.contains(field))
                        .map(|(key, value)| format!("unexpected field {key}: {value:?}")),
                )
                .collect::<Vec<_>>()
                .join("\n");
            vec![format!(
                "{} {}\n{differences}\nexpected fields: {expected:#?}\nactual fields: {actual:#?}",
                request.method, request.path
            )]
        };
        (errors, self.outcome)
    }
}

impl SubscriptionRequest {
    pub(super) fn at(self, at: i64) -> Self {
        Self::Reviewed(Box::new(self), at)
    }

    fn fields(&self) -> Vec<(String, String)> {
        if let Self::Reviewed(request, at) = self {
            let mut fields = request.fields();
            fields.push(("proration_date".into(), at.to_string()));
            return fields;
        }
        let (mode, items) = match self {
            Self::Change(mode, items) => (mode, items),
            Self::Reviewed(_, _) => unreachable!(),
            Self::CancelAtPeriodEnd => return vec![("cancel_at_period_end".into(), "true".into())],
            Self::CancelImmediately(_) => {
                return vec![
                    ("invoice_now".into(), "false".into()),
                    ("prorate".into(), "false".into()),
                ];
            }
        };
        let (proration, payment) = match mode {
            ChangeMode::Upgrade => ("always_invoice", "pending_if_incomplete"),
            ChangeMode::Prorated => ("create_prorations", "allow_incomplete"),
        };
        let mut fields = vec![
            ("proration_behavior".into(), proration.into()),
            ("payment_behavior".into(), payment.into()),
        ];
        for (index, item) in items.iter().enumerate() {
            match item {
                ItemChange::Update(id, price, quantity) => {
                    fields.push((format!("items[{index}][id]"), (*id).into()));
                    fields.push((format!("items[{index}][price]"), (*price).into()));
                    fields.push((format!("items[{index}][quantity]"), quantity.to_string()));
                }
                ItemChange::Add(price, quantity) => {
                    fields.push((format!("items[{index}][price]"), (*price).into()));
                    fields.push((format!("items[{index}][quantity]"), quantity.to_string()));
                }
                ItemChange::Remove(id) => {
                    fields.push((format!("items[{index}][id]"), (*id).into()));
                    fields.push((format!("items[{index}][deleted]"), "true".into()));
                }
            }
        }
        fields
    }
}
