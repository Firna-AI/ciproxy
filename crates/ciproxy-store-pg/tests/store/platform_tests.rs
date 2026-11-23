//! The platform of a label and of a machine stays as it was at creation.

use ciproxy_protocol::platform::{Platform, Virtualization};
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::gateway::Contact;
use ciproxy_store_interface::labels::{Label, LabelPlace, WhenNothingFits};
use ciproxy_store_interface::vm_size::VmSize;
use uuid::Uuid;

use super::gateway_support::registration;
use super::support::{Database, at, hash};

#[test]
fn keeps_the_platform_of_a_label_after_an_update() {
    let database = Database::new();
    let workspace = database.workspace();
    let mut label = Label {
        id: Uuid::new_v4(),
        name: "ciproxy-macos".to_owned(),
        platform: Platform::MacosArm64,
        size: VmSize {
            vcpus: 2,
            memory_gb: 8,
            disk_gb: 20,
        },
        order: vec![LabelPlace::Machine],
        when_nothing_fits: WhenNothingFits::Wait,
    };
    database.store.create_label(workspace.id, &label).unwrap();
    label.platform = Platform::LinuxX64;
    label.size.vcpus = 4;
    database.store.update_label(workspace.id, &label).unwrap();
    let saved = database.store.labels(workspace.id).unwrap().remove(0);
    assert_eq!(
        (saved.platform, saved.size.vcpus),
        (Platform::MacosArm64, 4)
    );
}

#[test]
fn keeps_the_platform_of_registration_and_refuses_a_contact_with_another() {
    let database = Database::new();
    let workspace = database.workspace();
    let token_hash = hash();
    database
        .store
        .issue_install_token(workspace.id, Uuid::new_v4(), &token_hash, at())
        .unwrap();
    let mut registration = registration(workspace.id, token_hash);
    registration.contact.report.hardware.platform = Platform::MacosArm64;
    registration.contact.report.hardware.virtualization = Virtualization::EntitlementMissing;
    assert!(database.store.register_machine(&registration).unwrap());
    let machine = database.store.machines(workspace.id).unwrap().remove(0);
    assert_eq!(
        (machine.platform, machine.virtualization),
        (Platform::MacosArm64, Virtualization::EntitlementMissing)
    );

    let mut linux = registration.contact.clone();
    linux.report.hardware.platform = Platform::LinuxX64;
    assert!(matches!(
        database
            .store
            .record_contact(workspace.id, registration.machine_id, &linux),
        Err(Error::InvalidValue)
    ));
    assert!(matches!(
        database.store.connect_machine(
            workspace.id,
            registration.machine_id,
            &registration.credential_hash,
            &Contact {
                session_id: Uuid::new_v4(),
                ..linux
            },
        ),
        Err(Error::InvalidValue)
    ));

    let mut changed = database.store.machines(workspace.id).unwrap().remove(0);
    changed.platform = Platform::LinuxX64;
    database
        .store
        .update_machine(workspace.id, &changed)
        .unwrap();
    let saved = database.store.machines(workspace.id).unwrap().remove(0);
    assert_eq!(saved.platform, Platform::MacosArm64);
    let report = database.store.machine_snapshots(workspace.id).unwrap()[0]
        .report
        .clone()
        .unwrap()
        .report;
    assert_eq!(report.hardware.platform, Platform::MacosArm64);
}
