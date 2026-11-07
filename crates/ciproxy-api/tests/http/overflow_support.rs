//! Real Boat service composition and controlled runner inputs for HTTP tests.

use std::sync::Arc;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_api::runtime::Runtime;
use ciproxy_overflow::api::{OverflowConfig, StartRequest};
use ciproxy_overflow::service::{OverflowDependencies, OverflowService};
use ciproxy_protocol::identifiers::{LabelName, OrganizationId};
use ciproxy_protocol::resources::VmSize;
use ciproxy_protocol::secret::Secret;
use ciproxy_scaler::actions::{Destination, StartReason, StartRunner};
use ciproxy_scaler::boat::BoatSizeId;
use unimock::Unimock;
use uuid::Uuid;

use super::fixture::Fixture;
use super::overflow_ports::{Factory, Ports};

pub(super) fn service(fixture: &mut Fixture) -> Arc<Ports> {
    let ports = Ports::new();
    *ports.now.lock().unwrap() = fixture.runtime.now();
    fixture.state.overflow = Some(Arc::new(OverflowService::new(
        OverflowConfig {
            deployment: "api-integration".to_owned(),
            shared_snapshot: "shared-image".to_owned(),
            root_sizing: RootSizing {
                image_root_bytes: fixture.state.config.image_disk_bytes,
            },
            exit_report_url: Secret::new("https://app.ciproxy.example/hooks/boat/runner-exit"),
        },
        OverflowDependencies {
            store: fixture.database.store.clone(),
            cipher: ports.clone(),
            factory: Arc::new(Factory(ports.clone())),
            runners: Arc::new(Unimock::new(())),
            clock: ports.clone(),
            tokens: ports.clone(),
        },
    )));
    ports
}

pub(super) async fn start(fixture: &Fixture) {
    let label = fixture
        .database
        .store
        .labels(fixture.workspace)
        .unwrap()
        .remove(0);
    let set = fixture
        .database
        .store
        .scale_sets(fixture.workspace, fixture.organization.id)
        .unwrap()
        .remove(0);
    let overflow = fixture.state.overflow.as_ref().unwrap();
    overflow
        .start(
            fixture.workspace,
            StartRequest {
                delivery_id: Uuid::new_v4(),
                runner_id: Uuid::new_v4(),
                organization_id: fixture.organization.id,
                label_id: label.id,
                scale_set_id: set.id,
                action: StartRunner {
                    organization_id: OrganizationId(fixture.organization.id.to_string()),
                    label: LabelName(label.name),
                    requested_size: VmSize {
                        vcpu: 2,
                        memory_bytes: 8_000_000_000,
                        disk_bytes: 20_000_000_000,
                    },
                    size: VmSize {
                        vcpu: 8,
                        memory_bytes: 16_000_000_000,
                        disk_bytes: 111_000_000_000,
                    },
                    destination: Destination::Boat(BoatSizeId("large".to_owned())),
                    reason: StartReason::NoEligibleMachine,
                },
                registration: Secret::new("fake-registration"),
            },
        )
        .await
        .unwrap();
}
