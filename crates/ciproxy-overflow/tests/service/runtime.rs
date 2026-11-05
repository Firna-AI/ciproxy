//! Strict fact and cryptography mocks shared by real-store service scenarios.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use ciproxy_overflow::clock::ClockMock;
use ciproxy_overflow::crypto::{KeyCipherMock, SecretPurpose};
use ciproxy_overflow::secrets::{SecretBytes, TokenSourceMock};
use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use super::authority_support::disable;
use super::harness::Runtime;
use super::recording::Capture;

pub(super) fn build(
    extra: impl unimock::Clause,
    state: Arc<Mutex<Capture>>,
    clock: Arc<Mutex<DateTime<Utc>>>,
    kind: Runtime,
) -> Arc<Unimock> {
    let sealing = state.clone();
    let now = ClockMock::now
        .some_call(matching!())
        .answers_arc(Arc::new(move |_| *clock.lock().unwrap()));
    let tokens = AtomicUsize::new(0);
    let token = TokenSourceMock::token
        .some_call(matching!())
        .answers_arc(Arc::new(move |_| {
            let number = tokens.fetch_add(1, Ordering::SeqCst);
            let token = if number == 0 {
                "callback-token-for-one-sandbox".to_owned()
            } else {
                format!("callback-token-for-one-sandbox-{number}")
            };
            Secret::new(token)
        }));
    let seal = KeyCipherMock::seal
        .some_call(matching!(_, _))
        .answers_arc(Arc::new(move |_, context, bytes| {
            let purpose = match context.purpose {
                SecretPurpose::BoatKey => 1,
                SecretPurpose::Launch => 2,
            };
            let mut state = sealing.lock().unwrap();
            state.contexts.push(context);
            state.sealed.insert((context.id, purpose), bytes);
            Ok([context.id.as_bytes().as_slice(), &[purpose]].concat())
        }));
    let open = KeyCipherMock::open
        .some_call(matching!(_, _))
        .answers_arc(Arc::new(move |_, context, _| {
            if context.purpose == SecretPurpose::BoatKey {
                let withdraw = {
                    let mut capture = state.lock().unwrap();
                    capture.key_opens += 1;
                    if capture
                        .disable_on_key_open
                        .as_ref()
                        .is_some_and(|(at, _)| *at == capture.key_opens)
                    {
                        capture.disable_on_key_open.take().map(|(_, store)| store)
                    } else {
                        None
                    }
                };
                if let Some(store) = withdraw {
                    disable(&store);
                }
                return Ok(SecretBytes::new(b"account-key".to_vec()));
            }
            Ok(state
                .lock()
                .unwrap()
                .sealed
                .get(&(context.id, 2))
                .expect("sealed launch")
                .clone())
        }));
    Arc::new(match kind {
        Runtime::KeyCheck => Unimock::new(extra),
        Runtime::Connect => Unimock::new((extra, now, seal)),
        Runtime::Recover => Unimock::new((extra, open)),
        Runtime::Read => Unimock::new((extra, now, open)),
        Runtime::Authority => Unimock::new((extra, now, seal, open)),
        Runtime::Start => Unimock::new((extra, now, token, seal, open)),
    })
}
