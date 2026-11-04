//! Runtime mode actions derive their next state under the machine state lock.

use ciproxy_protocol::runners::RunnerState;

use crate::controller::Controller;
use crate::error::{Error, Result};
use crate::state::Mode;

pub(crate) enum ModeAction {
    Drain,
    Resume,
    Remove,
}

impl Controller {
    pub(crate) async fn change_mode(&self, action: ModeAction) -> Result<()> {
        {
            let mut state = self.state.lock().await;
            let mode = match (state.mode, action) {
                (Mode::Removed, ModeAction::Resume) => return Err(Error::Revoked),
                (Mode::Removed, ModeAction::Drain) => return Ok(()),
                (Mode::Removed, ModeAction::Remove) => Mode::Removed,
                (_, ModeAction::Drain) => Mode::Draining,
                (_, ModeAction::Resume) => Mode::Active,
                (_, ModeAction::Remove) => Mode::Removed,
            };
            if state.mode != mode {
                let mut next = state.clone();
                next.mode = mode;
                self.store.save(&next)?;
                *state = next;
            }
        }
        self.enforce_mode().await
    }

    pub(crate) async fn enforce_mode(&self) -> Result<()> {
        let runners = {
            let state = self.state.lock().await;
            state
                .runners
                .iter()
                .filter(|vm| {
                    !matches!(vm.report.state, RunnerState::Running { .. })
                        && (state.mode != Mode::Active || vm.report.state == RunnerState::Stopping)
                })
                .cloned()
                .collect::<Vec<_>>()
        };
        let mut failure = None;
        for vm in runners {
            if let Err(error) = self.request_idle_stop(&vm).await {
                tracing::warn!(error = %error, "idle VM shutdown pending; reconciliation will retry");
                if failure.is_none() {
                    failure = Some(error);
                }
            }
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
