//! Wire pure Boat actions to the existing durable overflow executor.

use ciproxy_github::registrations::RegistrationRequest;
use ciproxy_overflow::api::{StartOutcome, StartRequest};
use ciproxy_protocol::identifiers::OrganizationId;
use ciproxy_protocol::secret::Secret;
use ciproxy_scaler::actions::StartRunner;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow_runtime::OverflowRunnerIntent;
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use ciproxy_store_interface::runtime_results::{OverflowRunnerResult, OverflowRunnerWrite};
use ciproxy_store_interface::scale_sets::RegistrationTarget;
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;
use crate::runtime_tasks::Work;

impl MachineRuntime {
    pub(crate) async fn start_boat(
        &self,
        input: &RoutingSnapshot,
        mut action: StartRunner,
    ) -> Result<()> {
        let overflow = self.overflow.as_ref().ok_or(Error::StateChanged)?;
        let organization: Uuid = action.target.organization_id.0.parse().internal()?;
        let github_organization = input
            .organizations
            .iter()
            .find(|candidate| candidate.id == organization)
            .ok_or(Error::StateChanged)?
            .github_id;
        action.organization_id = OrganizationId(github_organization.to_string());
        let label = input
            .labels
            .iter()
            .find(|label| label.id.to_string() == action.target.label_id)
            .ok_or(Error::StateChanged)?;
        let workspace = input.workspace_id;
        let set = input
            .scale_sets
            .iter()
            .find(|set| {
                set.id.to_string() == action.target.scale_set_id
                    && set.organization_id == organization
                    && set.label_id == label.id
                    && set.github_id == action.target.github_id
            })
            .ok_or(Error::StateChanged)?;
        let intent = OverflowRunnerIntent {
            provider: OverflowProvider::Boat,
            registration_scale_set_id: None,
            delivery_id: self.clock.uuid(),
            runner_id: self.clock.uuid(),
            organization_id: organization,
            label_id: label.id,
            label_name: label.name.clone(),
            scale_set_id: set.id,
            requested_size: action.requested_size,
            size: action.size,
            required_root_disk_bytes: self
                .root_sizing()
                .root_bytes(action.requested_size.disk_bytes)
                .ok_or(Error::StateChanged)?,
            github_id: None,
            created_at: self.clock.now(),
        };
        let saved = intent.clone();
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.create_overflow_runner_intent(workspace, &saved))
            .await
            .internal()?
            .internal()?;
        let store = self.store.clone();
        let runner_id = intent.runner_id;
        let expected =
            tokio::task::spawn_blocking(move || store.overflow_runner_guard(workspace, runner_id))
                .await
                .internal()?
                .internal()?
                .ok_or(Error::StateChanged)?;
        let request = RegistrationRequest {
            workspace_id: workspace,
            target: RegistrationTarget::from(set),
        };
        let label_id = label.id;
        let scale_set_id = set.id;
        let overflow = overflow.clone();
        let runtime = self.worker();
        self.dispatch(Work::Create(runner_id), async move {
            let registration = runtime
                .registrations
                .create(request, &format!("ciproxy-{runner_id}"))
                .await
                .internal()?;
            let target = registration.target;
            let github_id = registration.runner_id;
            let store = runtime.store.clone();
            let accepted = tokio::task::spawn_blocking(move || {
                store.apply_overflow_runner_result(
                    workspace,
                    &expected,
                    OverflowRunnerResult::Registered { target, github_id },
                )
            })
            .await
            .internal()?
            .internal()?;
            if !matches!(accepted, OverflowRunnerWrite::Applied(_)) {
                return runtime.cleanup_late_registration(workspace, runner_id, target, github_id);
            }
            let boat = runtime.boat_operation(workspace).await;
            let _boat = boat.lock().await;
            let outcome = overflow
                .start(
                    workspace,
                    StartRequest {
                        delivery_id: intent.delivery_id,
                        runner_id,
                        organization_id: organization,
                        label_id,
                        scale_set_id,
                        action,
                        registration: Secret::new(registration.configuration.expose_secret()),
                    },
                )
                .await
                .internal()?;
            if let StartOutcome::Pending(_) | StartOutcome::Started(_) = outcome {
                let store = runtime.store.clone();
                tokio::task::spawn_blocking(move || {
                    store.register_overflow_runner(workspace, runner_id, github_id)
                })
                .await
                .internal()?
                .internal()?;
            }
            Ok(())
        })
    }
}
