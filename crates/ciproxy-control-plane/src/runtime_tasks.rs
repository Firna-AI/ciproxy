//! Owned remote work with deduplication, joined errors, and explicit quiescence.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock, Weak};

use tokio::runtime::Handle;
use tokio::sync::{Notify, mpsc, oneshot};
use tokio::task::{Id, JoinSet};
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime_inbox::Event;
use crate::tasks::OwnedTask;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Work {
    Create(Uuid),
    Retire(Uuid),
    Preparation(Uuid),
    BoatRefresh(Uuid),
    BoatCleanup(Uuid),
    BoatRecovery(Uuid),
    BoatRetire(Uuid),
    Takeout(Uuid),
}

type Pending = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

enum Request {
    Run(Work, Pending),
    Shutdown(oneshot::Sender<()>),
}

#[derive(Default)]
struct State {
    keys: HashSet<Work>,
    error: Option<(Work, Error)>,
    queued: usize,
    stopped: bool,
}

pub(crate) struct RuntimeTasks {
    state: Mutex<State>,
    changed: Notify,
    sender: mpsc::UnboundedSender<Request>,
    owner: Mutex<Option<OwnedTask<()>>>,
    pub(crate) handle: Handle,
    pub(crate) inbox: OnceLock<mpsc::Sender<Event>>,
    pub(crate) inbox_owner: Mutex<Option<OwnedTask<()>>>,
}

impl RuntimeTasks {
    pub(crate) fn new() -> Arc<Self> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let tasks = Arc::new(Self {
            state: Mutex::new(State::default()),
            changed: Notify::new(),
            sender,
            owner: Mutex::new(None),
            handle: Handle::current(),
            inbox: OnceLock::new(),
            inbox_owner: Mutex::new(None),
        });
        let task = tasks
            .handle
            .spawn(supervise(Arc::downgrade(&tasks), receiver));
        *tasks
            .owner
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(OwnedTask(task));
        tasks
    }

    pub(crate) fn dispatch(&self, key: Work, future: Pending) -> Result<()> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.stopped {
            return Err(Error::StateChanged);
        }
        if !state.keys.insert(key) {
            return Ok(());
        }
        if self.sender.send(Request::Run(key, future)).is_err() {
            state.keys.remove(&key);
            return Err(Error::StateChanged);
        }
        Ok(())
    }

    pub(crate) fn contains(&self, key: Work) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .keys
            .contains(&key)
    }

    pub(crate) fn queue_added(&self) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.stopped {
            return false;
        }
        state.queued += 1;
        true
    }

    pub(crate) fn queue_finished(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.queued = state.queued.saturating_sub(1);
        self.changed.notify_waiters();
    }

    pub(crate) async fn wait_for(&self, key: Work) {
        loop {
            let notified = self.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if !self.contains(key) {
                return;
            }
            notified.await;
        }
    }

    pub(crate) async fn quiesce(&self) -> Result<()> {
        match self.drain().await {
            Some((_, error)) => Err(error),
            None => Ok(()),
        }
    }

    pub(crate) async fn lifecycle_quiesce(&self) -> Result<()> {
        match self.drain().await {
            Some((_, error @ Error::TaskJoin(_)))
            | Some((_, error @ Error::StateChanged))
            | Some((Work::Takeout(_), error)) => Err(error),
            Some(_) | None => Ok(()),
        }
    }

    async fn drain(&self) -> Option<(Work, Error)> {
        loop {
            let notified = self.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            {
                let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
                if state.keys.is_empty() && state.queued == 0 {
                    return state.error.take();
                }
            }
            notified.await;
        }
    }

    pub(crate) fn cancel(&self) {
        if let Some(task) = self
            .inbox_owner
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
        {
            task.0.abort();
        }
        if let Some(task) = self
            .owner
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
        {
            task.0.abort();
        }
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.stopped = true;
        state.keys.clear();
        state.queued = 0;
        self.changed.notify_waiters();
    }

    pub(crate) async fn shutdown(&self) -> Result<()> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .stopped = true;
        let inbox = self
            .inbox_owner
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        if let Some(mut task) = inbox {
            task.0.abort();
            if let Err(error) = (&mut task.0).await
                && !error.is_cancelled()
            {
                tracing::warn!(%error,"agent queue shutdown join failed");
            }
        }
        let (sender, receiver) = oneshot::channel();
        if self.sender.send(Request::Shutdown(sender)).is_err() {
            return Ok(());
        }
        receiver.await.internal()?;
        let owner = self
            .owner
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        if let Some(mut task) = owner {
            (&mut task.0).await.internal()?;
        }
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.stopped = true;
        state.keys.clear();
        state.queued = 0;
        self.changed.notify_waiters();
        Ok(())
    }

    fn completed(&self, key: Work, result: Result<()>) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.keys.remove(&key);
        if let Err(error) = result {
            tracing::warn!(?key, %error, "runtime task failed; durable recovery will retry");
            let incoming = priority(key, &error);
            if state
                .error
                .as_ref()
                .is_none_or(|(key, error)| priority(*key, error) <= incoming)
            {
                state.error = Some((key, error));
            }
        }
        self.changed.notify_waiters();
    }
}

async fn supervise(owner: Weak<RuntimeTasks>, mut receiver: mpsc::UnboundedReceiver<Request>) {
    let mut tasks = JoinSet::new();
    let mut keys: HashMap<Id, Work> = HashMap::new();
    loop {
        tokio::select! {
            request = receiver.recv() => match request {
                Some(Request::Run(key, future)) => { keys.insert(tasks.spawn(future).id(), key); }
                Some(Request::Shutdown(done)) => {
                    receiver.close();
                    tasks.abort_all();
                    while let Some(result) = tasks.join_next_with_id().await {
                        if let Err(error) = result && !error.is_cancelled() {
                            tracing::warn!(%error,"runtime shutdown task join failed");
                        }
                    }
                    let _ = done.send(());
                    return;
                }
                None => return,
            },
            result = tasks.join_next_with_id(), if !tasks.is_empty() => {
                let Some(owner) = owner.upgrade() else { return; };
                match result {
                    Some(Ok((id, result))) => {
                        if let Some(key) = keys.remove(&id) { owner.completed(key, result); }
                    }
                    Some(Err(error)) => {
                        if let Some(key) = keys.remove(&error.id()) {
                            owner.completed(key, Err(Error::TaskJoin(error)));
                        }
                    }
                    None => (),
                }
            }
        }
    }
}

fn priority(key: Work, error: &Error) -> u8 {
    match (key, error) {
        (_, Error::TaskJoin(_)) => 4,
        (Work::Preparation(_), _) => 3,
        (Work::Takeout(_), _) => 2,
        (_, Error::StateChanged) => 1,
        _ => 0,
    }
}
