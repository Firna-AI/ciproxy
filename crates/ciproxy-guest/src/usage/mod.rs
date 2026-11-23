//! Start job sampling on the correct signal, without waiting on report posts.

mod cpu;
#[cfg(target_os = "linux")]
pub(crate) mod linux;
#[cfg(any(target_os = "macos", test))]
pub(crate) mod macos;
mod series;
mod window;
pub(crate) mod worker_log;

use std::future::{Future, pending};
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::usage::JobUsage;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{Instant, Interval, MissedTickBehavior, interval_at};
use tracing::warn;

use crate::error::Result;

pub(crate) use self::cpu::CpuReading;
pub(crate) use self::window::Reading;
use self::window::Window;
use self::worker_log::{LogPages, WorkerLog};

const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
const REPORT_INTERVAL: Duration = Duration::from_secs(30);

/// The platform measurements and the stock worker start signal.
#[cfg_attr(test, unimock::unimock(api = SourceMock))]
pub(crate) trait Source: Send {
    /// The platform field of this source's usage report.
    fn platform(&self) -> Platform;
    /// Read the VM clock and counters without logging their values.
    fn read(&self) -> io::Result<Reading>;
    /// Check for a live worker below the runner process.
    fn worker_present(&self, runner_pid: u32) -> io::Result<bool>;
}

/// A cancellable Boat post.
pub(crate) type ReportFuture<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

/// The scoped Boat callback, held independently of the main control channel.
#[cfg_attr(test, unimock::unimock(api = ReporterMock))]
pub(crate) trait Reporter: Send + Sync {
    /// Post one whole-series sample report.
    fn post_samples(&self, message: GuestToAgent) -> ReportFuture<'_>;
}

/// Usage collection for one runner. No report is sent until its job starts.
pub(crate) struct Usage {
    source: Option<Box<dyn Source>>,
    reporter: Option<Arc<dyn Reporter>>,
    log: Option<WorkerLog>,
    task: Option<Sampling>,
}

impl Usage {
    /// Use the worker start signal only when the transport is Boat.
    pub(crate) fn new(
        source: Box<dyn Source>,
        reporter: Option<Arc<dyn Reporter>>,
        pages: Box<dyn LogPages>,
    ) -> Self {
        Self {
            source: Some(source),
            reporter,
            log: Some(WorkerLog::new(pages)),
            task: None,
        }
    }

    /// Exclude logs left by earlier workers before the listener can start.
    pub(crate) fn prepare_runner(&mut self) {
        if let Some(log) = &mut self.log
            && log.exclude_existing_pages().is_err()
        {
            warn!("runner step log unavailable");
            self.log = None;
        }
    }

    /// For Boat, begin looking for a worker below the newly started runner.
    pub(crate) fn runner_started(&mut self, pid: Option<u32>, runner_id: &RunnerId) {
        if let Some(reporter) = self.reporter.as_ref().cloned()
            && let Some(runner_pid) = pid
            && let Some(source) = self.source.take()
        {
            self.task = Some(Sampling::start(
                source,
                self.log.take(),
                Some(BoatJob {
                    runner_pid,
                    runner_id: runner_id.clone(),
                    reporter,
                }),
            ));
        }
    }

    /// Start own-machine sampling once at the trusted job handoff.
    pub(crate) fn handoff(&mut self) {
        if self.reporter.is_none()
            && let Some(source) = self.source.take()
        {
            self.task = Some(Sampling::start(source, self.log.take(), None));
        }
    }

    /// Stop sampling and return the collected points, without waiting on HTTP.
    pub(crate) async fn finish(&mut self) -> Option<JobUsage> {
        self.task.take()?.finish().await
    }
}

struct BoatJob {
    runner_pid: u32,
    runner_id: RunnerId,
    reporter: Arc<dyn Reporter>,
}

struct Sampling {
    stop: Option<oneshot::Sender<()>>,
    task: JoinHandle<Option<JobUsage>>,
}

impl Sampling {
    fn start(source: Box<dyn Source>, log: Option<WorkerLog>, boat: Option<BoatJob>) -> Self {
        let (stop, stopped) = oneshot::channel();
        Self {
            stop: Some(stop),
            task: tokio::spawn(collect(source, log, boat, stopped)),
        }
    }

    async fn finish(mut self) -> Option<JobUsage> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        match (&mut self.task).await {
            Ok(usage) => usage,
            Err(_) => {
                warn!("job usage sampler could not finish");
                None
            }
        }
    }
}

impl Drop for Sampling {
    fn drop(&mut self) {
        self.task.abort();
    }
}

type Post = Pin<Box<dyn Future<Output = ()> + Send>>;

async fn collect(
    source: Box<dyn Source>,
    mut log: Option<WorkerLog>,
    boat: Option<BoatJob>,
    mut stop: oneshot::Receiver<()>,
) -> Option<JobUsage> {
    if let Some(job) = &boat {
        let mut search = interval(Instant::now(), SAMPLE_INTERVAL);
        loop {
            tokio::select! {
                biased;
                _ = &mut stop => return None,
                _ = search.tick() => match source.worker_present(job.runner_pid) {
                    Ok(true) => break,
                    Ok(false) => {},
                    Err(_) => warn!("runner worker could not be observed"),
                },
            }
        }
    }

    let mut window = Window::new(source.platform());
    let started = Instant::now();
    window.capture(&*source, started.elapsed());
    let mut ticks = interval(started + SAMPLE_INTERVAL, SAMPLE_INTERVAL);
    let mut reports = interval(started + REPORT_INTERVAL, REPORT_INTERVAL);
    let mut posting: Option<Post> = None;
    loop {
        tokio::select! {
            biased;
            _ = &mut stop => return report(&window, &mut log),
            _ = ticks.tick() => window.capture(&*source, started.elapsed()),
            () = wait_for_post(&mut posting) => posting = None,
            _ = reports.tick(), if boat.is_some() => {
                if let Some(job) = &boat
                    && let Some(usage) = report(&window, &mut log)
                {
                    if posting.is_some() {
                        warn!("job usage sample report could not be delivered");
                    }
                    let reporter = job.reporter.clone();
                    let message = GuestToAgent::UsageSamples {
                        runner_id: job.runner_id.clone(),
                        usage: usage.into(),
                    };
                    posting = Some(Box::pin(async move {
                        if reporter.post_samples(message).await.is_err() {
                            warn!("job usage sample report could not be delivered");
                        }
                    }));
                }
            }
        }
    }
}

fn report(window: &Window, log: &mut Option<WorkerLog>) -> Option<JobUsage> {
    let mut usage = window.report()?;
    if let Some(worker_log) = log {
        match worker_log.report() {
            Ok(steps) => usage.steps = steps,
            Err(_) => {
                warn!("runner step log unavailable");
                *log = None;
            }
        }
    }
    Some(usage)
}

fn interval(first: Instant, period: Duration) -> Interval {
    let mut ticks = interval_at(first, period);
    ticks.set_missed_tick_behavior(MissedTickBehavior::Skip);
    ticks
}

async fn wait_for_post(posting: &mut Option<Post>) {
    match posting {
        Some(post) => post.await,
        None => pending().await,
    }
}
