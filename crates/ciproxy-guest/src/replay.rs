//! Keep assignment and completed preparation available across channel shutdown.

use ciproxy_protocol::guest::GuestToAgent;

#[derive(Default)]
pub(crate) struct Observations {
    hello: Option<GuestToAgent>,
    current: Option<GuestToAgent>,
    prepared: Option<GuestToAgent>,
}

impl Observations {
    pub(crate) fn record(&mut self, message: &GuestToAgent) -> bool {
        match message {
            GuestToAgent::RunnerReady { .. }
                if matches!(self.current, Some(GuestToAgent::JobStarting { .. })) =>
            {
                return false;
            }
            GuestToAgent::Hello { .. } => self.hello = Some(message.clone()),
            GuestToAgent::RunnerConfigured { .. } => {
                if let Some(GuestToAgent::Hello {
                    configuration_received,
                    ..
                }) = &mut self.hello
                {
                    *configuration_received = true;
                }
            }
            GuestToAgent::JobPrepared { .. } => self.prepared = Some(message.clone()),
            _ => self.current = Some(message.clone()),
        }
        true
    }

    pub(crate) fn replay(&self) -> Vec<GuestToAgent> {
        [
            self.hello.clone(),
            self.current.clone(),
            self.prepared.clone(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

#[cfg(test)]
#[path = "_tests_/replay_tests.rs"]
mod replay_tests;
