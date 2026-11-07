//! Real-store runtime composition for overflow integration scenarios.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicUsize, Ordering};

use chrono::{DateTime, Utc};
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_control_plane::boat_authority::BoatRunnerAuthority;
use ciproxy_control_plane::runtime::MachineRuntime;
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_gateway::types::Delivery;
use ciproxy_github::registrations::{
    RegistrationRetirement, RunnerRegistrations, RunnerRegistrationsMock,
};
use ciproxy_github::scale_sets::JitRegistration;
use ciproxy_overflow::api::{Overflow, OverflowConfig};
use ciproxy_overflow::service::{OverflowDependencies, OverflowService};
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::resources::Slots;
use ciproxy_protocol::runners::RunnerState;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::github::{DemandPhase, GithubJob, RepositoryIdentity, RunFacts};
use ciproxy_store_interface::runners;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::overflow_ports::{Factory, Ports};
use super::support::{Fixture, at};

pub(super) struct Harness {
    pub(super) fixture: Fixture,
    pub(super) ports: Arc<Ports>,
    pub(super) runtime: MachineRuntime,
    pub(super) service: Arc<dyn Overflow>,
    pub(super) busy: Arc<AtomicBool>,
    pub(super) remove_app_on_retire: Arc<AtomicBool>,
    pub(super) own_starts: Arc<AtomicUsize>,
}

impl Harness {
    pub(super) async fn new(eligible: bool, occupied: bool) -> Self {
        let fixture = Fixture::new();
        fixture
            .store
            .update_runner(
                fixture.workspace,
                fixture.organization,
                fixture.intent.runner.id,
                runners::RunnerState::Stopped,
                Some(47),
            )
            .unwrap();
        let mut label = fixture.store.labels(fixture.workspace).unwrap().remove(0);
        label.size.disk_gb = 20;
        label.allow_boat = true;
        fixture
            .store
            .update_label(fixture.workspace, &label)
            .unwrap();
        let mut contact = fixture.contact.clone();
        contact.report.capacity = Slots(1);
        if occupied {
            contact.report.runners[0].state = RunnerState::Running {
                job_id: JobId("existing-job".to_owned()),
            };
            fixture
                .store
                .update_runner(
                    fixture.workspace,
                    fixture.organization,
                    fixture.intent.runner.id,
                    runners::RunnerState::Busy,
                    Some(47),
                )
                .unwrap();
        } else {
            contact.report.runners.clear();
        }
        fixture
            .store
            .record_contact(fixture.workspace, fixture.machine, &contact)
            .unwrap();
        fixture
            .store
            .set_machine_draining(fixture.workspace, fixture.machine, !eligible)
            .unwrap();
        let ports = Ports::new();
        let busy = Arc::new(AtomicBool::new(false));
        let race = busy.clone();
        let remove_app_on_retire = Arc::new(AtomicBool::new(false));
        let removal = remove_app_on_retire.clone();
        let removal_store = fixture.store.clone();
        let workspace = fixture.workspace;
        let organization = fixture.organization;
        let ids = AtomicI64::new(100);
        let registrations: Arc<dyn RunnerRegistrations> = Arc::new(
            Unimock::new((
                RunnerRegistrationsMock::create
                    .some_call(matching!(_, _))
                    .answers_arc(Arc::new(move |_, _, _| {
                        Ok(JitRegistration {
                            runner_id: ids.fetch_add(1, Ordering::SeqCst),
                            configuration: "fake-jit".into(),
                        })
                    })),
                RunnerRegistrationsMock::retire
                    .some_call(matching!(_, _, _))
                    .answers_arc(Arc::new(move |_, _, _, _| {
                        if removal.swap(false, Ordering::SeqCst) {
                            removal_store
                                .record_app_removal(workspace, organization, at())
                                .unwrap();
                        }
                        Ok(if race.load(Ordering::SeqCst) {
                            RegistrationRetirement::Busy { id: 100 }
                        } else {
                            RegistrationRetirement::Removed { id: 100 }
                        })
                    })),
            ))
            .no_verify_in_drop(),
        );
        let service: Arc<dyn Overflow> = Arc::new(OverflowService::new(
            OverflowConfig {
                deployment: "runtime-integration".to_owned(),
                shared_snapshot: "shared-image".to_owned(),
                root_sizing: RootSizing::default(),
                exit_report_url: Secret::new("https://app.ciproxy.example/hooks/boat/runner-exit"),
            },
            OverflowDependencies {
                store: fixture.store.clone(),
                cipher: ports.clone(),
                factory: Arc::new(Factory(ports.clone())),
                runners: Arc::new(BoatRunnerAuthority {
                    store: fixture.store.clone(),
                    registrations: registrations.clone(),
                    root_sizing: RootSizing::default(),
                    clock: ports.clone(),
                }),
                clock: ports.clone(),
                tokens: ports.clone(),
            },
        ));
        service
            .connect(
                fixture.workspace,
                Uuid::new_v4(),
                Secret::new("fake-boat-key"),
                "personal".to_owned(),
                200_000_000,
            )
            .await
            .unwrap();
        let own_starts = Arc::new(AtomicUsize::new(0));
        let starts = own_starts.clone();
        let gateway = Arc::new(
            Unimock::new(
                MachineGatewayMock::send
                    .some_call(matching!(_, _, _))
                    .answers_arc(Arc::new(move |_, _, _, message| {
                        if matches!(message, ControlMessage::StartRunner { .. }) {
                            starts.fetch_add(1, Ordering::SeqCst);
                        }
                        Ok(Delivery::Connected)
                    })),
            )
            .no_verify_in_drop(),
        );
        let mut runtime = MachineRuntime::new(
            fixture.store.clone(),
            gateway,
            registrations.clone(),
            Arc::new(Unimock::new(())),
            ports.clone(),
            None,
            Arc::new(Unimock::new(())),
        );
        runtime.overflow = Some(service.clone());
        runtime.boat_retirement = Some(Arc::new(BoatRunnerAuthority {
            store: fixture.store.clone(),
            registrations: registrations.clone(),
            root_sizing: RootSizing::default(),
            clock: ports.clone(),
        }));
        Self {
            fixture,
            ports,
            runtime,
            service,
            busy,
            remove_app_on_retire,
            own_starts,
        }
    }

    pub(super) fn queued(&self) -> GithubJob {
        self.queued_for(
            self.fixture.intent.runner.label_id,
            self.fixture.intent.runner.scale_set_id,
        )
    }

    pub(super) fn queued_for(&self, label_id: Uuid, scale_set_id: Uuid) -> GithubJob {
        self.queued_for_at(label_id, scale_set_id, at())
    }

    pub(super) fn queued_at(&self, assigned_at: DateTime<Utc>) -> GithubJob {
        self.queued_for_at(
            self.fixture.intent.runner.label_id,
            self.fixture.intent.runner.scale_set_id,
            assigned_at,
        )
    }

    fn queued_for_at(
        &self,
        label_id: Uuid,
        scale_set_id: Uuid,
        assigned_at: DateTime<Utc>,
    ) -> GithubJob {
        let job = GithubJob {
            id: Uuid::new_v4(),
            organization_id: self.fixture.organization,
            scale_set_id,
            label_id,
            actions_job_id: None,
            request_id: 0,
            run_id: 7,
            repository: "org/repo".to_owned(),
            name: "build".to_owned(),
            workflow_ref: "ci.yml".to_owned(),
            phase: DemandPhase::Assigned,
            result: None,
            assigned_at: Some(assigned_at),
            started_at: None,
            completed_at: None,
            runner_github_id: None,
            facts: RunFacts {
                id: 7,
                repository: RepositoryIdentity {
                    id: 8,
                    full_name: "org/repo".to_owned(),
                },
                event: "push".to_owned(),
                head_branch: Some("main".to_owned()),
                head_sha: "fixture".to_owned(),
                head_repository: None,
            },
        };
        self.fixture
            .store
            .save_github_job(self.fixture.workspace, &job, assigned_at)
            .unwrap();
        job
    }
}
