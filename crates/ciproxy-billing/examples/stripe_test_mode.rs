//! Explicit, opt-in Stripe test-mode smoke run through the public billing API.

use std::env;
use std::error::Error;
use std::fs;
use std::process::ExitCode;
use std::sync::Arc;

use chrono::Utc;
use ciproxy_billing::access::{BillingAccess, Role};
use ciproxy_billing::alerts::HostedBillingAlerts;
use ciproxy_billing::clock::SystemClock;
use ciproxy_billing::error;
use ciproxy_billing::service::api::BillingApi;
use ciproxy_billing::service::paid_review::PaidReview;
use ciproxy_billing::service::runtime::{BillingService, Redirects};
use ciproxy_billing::service::view::{ChangeOutcome, CheckoutOutcome};
use ciproxy_billing::stripe::api::StripeApi;
use ciproxy_billing::stripe::http::client::HttpStripe;
use ciproxy_billing::webhook::SigningSecret;
use ciproxy_store_interface::billing::commands::CommandKind;
use ciproxy_store_interface::billing::state::{AddOns, Selection, Tier};
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_interface::workspaces::Workspace;
use ciproxy_store_pg::factory::PgStoreFactory;
use serde::{Deserialize, Serialize};
use tiny_http::{Response, Server};
use uuid::Uuid;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            eprintln!("Stripe test-mode step failed. Credentials and provider details are hidden.");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let action = env::args().nth(1).unwrap_or_else(|| "help".into());
    if action == "help" {
        println!(
            "Actions: provision, checkout, reconcile, upgrade, addons, portal, cancel, status, listen."
        );
        return Ok(());
    }
    let key = env::var("STRIPE_SECRET_KEY")?;
    if !key.starts_with("sk_test_") && !key.starts_with("rk_test_") {
        return Err("a Stripe test-mode key is required".into());
    }
    let provider: Arc<dyn StripeApi> = Arc::new(HttpStripe::new(key)?);
    if action == "provision" {
        provider.provision()?;
        println!("Four USD monthly prices and a restricted portal are ready.");
        return Ok(());
    }
    let database = env::var("DATABASE_URL")?;
    let workspace = env::var("CIPROXY_STRIPE_TEST_WORKSPACE")?.parse::<Uuid>()?;
    let secret = env::var("STRIPE_WEBHOOK_SECRET")?;
    let store = PgStoreFactory.connect(&database)?;
    if store.workspace(workspace)?.is_none() {
        store.create_workspace(&Workspace {
            id: workspace,
            name: "Stripe test-mode workspace".into(),
            delete_after: None,
        })?;
    }
    let access = BillingAccess {
        workspace_id: workspace,
        role: Role::Admin,
    };
    let origin = env::var("CIPROXY_STRIPE_TEST_RETURN_ORIGIN")
        .unwrap_or_else(|_| "https://example.com".into());
    let service = BillingService::new(
        store.clone(),
        provider,
        Arc::new(SystemClock),
        Arc::new(HostedBillingAlerts),
        SigningSecret::new(secret),
        Redirects {
            success_url: format!(
                "{origin}/checkout-return?workspace={{WORKSPACE_ID}}&session_id={{CHECKOUT_SESSION_ID}}"
            ),
            cancel_url: format!("{origin}/plans?workspace={{WORKSPACE_ID}}"),
            portal_return_url: format!("{origin}/billing?workspace={{WORKSPACE_ID}}"),
        },
    );
    let billing: &dyn BillingApi = &service;
    let selected = Selection {
        tier: Tier::Team,
        add_ons: AddOns {
            start_blocks: 2,
            machines: 1,
        },
    };
    match action.as_str() {
        "listen" => listen(billing)?,
        "checkout" => {
            let review = billing.review(access, selected)?;
            if review.monthly_cents != 8900 || review.today_cents != 8900 {
                return Err("incorrect Team review".into());
            }
            show_checkout(billing.open_checkout(access, selected)?);
            if let Some(checkout) = store.billing(workspace)?.checkout {
                println!("Checkout session: {}", checkout.session_id);
            }
        }
        "reconcile" => {
            let session = env::var("CIPROXY_STRIPE_TEST_SESSION")?;
            show_checkout(billing.reconcile_checkout(access, &session)?);
        }
        "upgrade" => {
            let quote_id = match store.billing(workspace)?.command {
                Some(command) => match command.kind {
                    CommandKind::Change {
                        quote: Some(quote), ..
                    } => quote.id,
                    _ => return Err("another billing command is in progress".into()),
                },
                None => {
                    PaidReview::review(
                        &service,
                        access,
                        Selection {
                            tier: Tier::Business,
                            ..selected
                        },
                        Utc::now(),
                    )?
                    .id
                }
            };
            let result = billing.change(access, quote_id)?;
            show_change(result);
        }
        "addons" => show_change(billing.change_add_ons(access, AddOns::default())?),
        "portal" => println!("Open the restricted portal: {}", billing.portal(access)?),
        "cancel" => {
            billing.move_to_free(access)?;
            println!("The paid subscription will end at its existing period end.");
        }
        "status" => {
            let record = billing.status(access)?;
            println!(
                "Tier: {:?}; state: {:?}; add-ons: {:?}",
                record.state.tier, record.state.status, record.state.add_ons
            );
            println!(
                "Paid period: {:?} to {:?}; scheduled end: {:?}",
                record.state.paid_period_start,
                record.state.paid_period_end,
                record.state.cancel_at
            );
            println!(
                "Failed payment: {:?}; next attempt: {:?}; ended banner until: {:?}",
                record.state.failed_at, record.state.next_retry_at, record.state.ended_banner_until
            );
        }
        _ => return Err("unknown test-mode action".into()),
    }
    Ok(())
}

fn show_checkout(outcome: CheckoutOutcome) {
    match outcome {
        CheckoutOutcome::Open { url } => println!("Complete payment in the browser: {url}"),
        CheckoutOutcome::Confirming => {
            println!("Payment is still confirming. Run reconcile again.")
        }
        CheckoutOutcome::Confirmed(state) => {
            println!("Confirmed {:?} with {:?}.", state.tier, state.add_ons)
        }
        CheckoutOutcome::Unfinished => {
            println!("Checkout did not finish. The tier stayed unchanged.")
        }
    }
}

fn show_change(outcome: ChangeOutcome) {
    match outcome {
        ChangeOutcome::Applied(state) => {
            println!("Applied {:?} with {:?}.", state.tier, state.add_ons)
        }
        ChangeOutcome::PaymentRequired { payment_url } => {
            println!("The old tier stays active. Payment or bank confirmation is required.");
            if let Some(url) = payment_url {
                println!("Open the Stripe payment page: {url}");
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
struct RecordedEvent {
    id: String,
    created: i64,
    #[serde(rename = "type")]
    event_type: String,
    data: RecordedData,
}

#[derive(Deserialize, Serialize)]
struct RecordedData {
    object: RecordedObject,
}

#[derive(Deserialize, Serialize)]
struct RecordedObject {
    id: String,
}

fn listen(billing: &dyn BillingApi) -> Result<(), Box<dyn Error>> {
    let server = match Server::http("127.0.0.1:4242") {
        Ok(server) => server,
        Err(_) => return Err("could not bind test listener".into()),
    };
    let directory = ".context/ciproxy-mvp/billing/stripe-test-mode";
    fs::create_dir_all(directory)?;
    println!("Stripe test listener: http://127.0.0.1:4242/hooks/stripe");
    for mut request in server.incoming_requests() {
        let signature = request
            .headers()
            .iter()
            .find(|header| header.field.equiv("Stripe-Signature"))
            .map(|header| header.value.as_str().to_owned())
            .unwrap_or_default();
        let mut raw = Vec::new();
        request.as_reader().read_to_end(&mut raw)?;
        let result = billing.webhook(&signature, &raw);
        let status = match result {
            Ok(outcome) => {
                let event: RecordedEvent = serde_json::from_slice(&raw)?;
                let bytes = serde_json::to_vec_pretty(&event)?;
                let name = Uuid::new_v4();
                fs::write(format!("{directory}/{name}.json"), bytes)?;
                println!("Signed event accepted: {outcome:?}. A sanitized recording was saved.");
                200
            }
            Err(error::Error::InvalidSignature) => 400,
            Err(_) => 503,
        };
        request.respond(Response::from_string("billing test listener").with_status_code(status))?;
    }
    Ok(())
}
