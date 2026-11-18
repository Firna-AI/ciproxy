//! Cancellation-safe gateway holds and lock-poll tracing for queued events.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use ciproxy_gateway::gateway::{MachineGateway, MachineGatewayMock};
use ciproxy_gateway::types::Delivery;
use ciproxy_protocol::agent::ControlMessage;
use tokio::sync::oneshot;
use tracing::Dispatch;
use tracing::span::Id;
use tracing::subscriber::NoSubscriber;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use unimock::{MockFn, Unimock, matching};

pub(super) struct HeldBudget {
    pub(super) gateway: Arc<dyn MachineGateway>,
    pub(super) entered: oneshot::Receiver<()>,
    pub(super) release: oneshot::Sender<()>,
}

impl HeldBudget {
    pub(super) fn new() -> Self {
        let (entered, waiting) = oneshot::channel();
        let (release, released) = oneshot::channel();
        let hold = Mutex::new(Some((entered, released)));
        let gateway = Arc::new(Unimock::new(
            MachineGatewayMock::send
                .some_call(matching!(_, _, ControlMessage::DockerBudget { .. }))
                .answers_arc(Arc::new(move |_, _, _, _| {
                    let blocked = hold.lock().unwrap().take();
                    if let Some((entered, released)) = blocked {
                        let _ = entered.send(());
                        let _ = released.blocking_recv();
                    }
                    Ok(Delivery::Connected)
                })),
        ));
        Self {
            gateway,
            entered: waiting,
            release,
        }
    }
}

pub(super) fn lock_observer() -> (Dispatch, oneshot::Receiver<()>, Arc<AtomicBool>) {
    let unobserved = Dispatch::new(NoSubscriber::default());
    let (entered, polled) = oneshot::channel();
    let closed = Arc::new(AtomicBool::new(false));
    let subscriber = Registry::default().with(LockPoll {
        entered: Mutex::new(Some(entered)),
        watched: Mutex::new(None),
        closed: closed.clone(),
        _unobserved: unobserved,
    });
    (Dispatch::new(subscriber), polled, closed)
}

/// Keep scoped and unobserved threads distinct in the shared callsite cache.
struct LockPoll {
    entered: Mutex<Option<oneshot::Sender<()>>>,
    watched: Mutex<Option<Id>>,
    closed: Arc<AtomicBool>,
    _unobserved: Dispatch,
}

impl Layer<Registry> for LockPoll {
    fn on_enter(&self, id: &Id, context: Context<'_, Registry>) {
        if context
            .metadata(id)
            .is_some_and(|metadata| metadata.name() == "agent_event_operation")
            && let Some(entered) = self.entered.lock().unwrap().take()
        {
            *self.watched.lock().unwrap() = Some(id.clone());
            let _ = entered.send(());
        }
    }

    fn on_close(&self, id: Id, _: Context<'_, Registry>) {
        if self.watched.lock().unwrap().as_ref() == Some(&id) {
            self.closed.store(true, Ordering::Release);
        }
    }
}
