//! Unit tests that span several modules of the composition root.

#[path = "_tests_/network_ownership_tests.rs"]
mod network_ownership_tests;

#[path = "_tests_/network_lifecycle_tests.rs"]
mod network_lifecycle_tests;

#[path = "_tests_/network_progress_tests.rs"]
mod network_progress_tests;

#[path = "_tests_/network_uninstall_tests.rs"]
mod network_uninstall_tests;

#[path = "_tests_/network_start_failure_tests.rs"]
mod network_start_failure_tests;

#[path = "_tests_/vm_root_tests.rs"]
mod vm_root_tests;

#[path = "_tests_/vm_root_failure_tests.rs"]
mod vm_root_failure_tests;

#[path = "_tests_/image_start_tests.rs"]
mod image_start_tests;

#[path = "_tests_/image_retry_tests.rs"]
mod image_retry_tests;

#[path = "_tests_/job_facts_tests.rs"]
mod job_facts_tests;

#[path = "_tests_/host_decision_tests.rs"]
mod host_decision_tests;

#[path = "_tests_/hosted_preparation_tests.rs"]
mod hosted_preparation_tests;

#[path = "_tests_/startup_tests.rs"]
mod startup_tests;

#[path = "_tests_/network_tests.rs"]
mod network_tests;

#[path = "_tests_/capacity_tests.rs"]
mod capacity_tests;

#[path = "_tests_/admission_tests.rs"]
mod admission_tests;

#[path = "_tests_/mac_admission_tests.rs"]
mod mac_admission_tests;

#[path = "_tests_/recovery_tests.rs"]
mod recovery_tests;

#[path = "_tests_/image_response_tests.rs"]
mod image_response_tests;

#[path = "_tests_/image_concurrency_tests.rs"]
mod image_concurrency_tests;

#[path = "_tests_/mode_reconcile_tests.rs"]
mod mode_reconcile_tests;

#[path = "_tests_/state_upgrade_tests.rs"]
mod state_upgrade_tests;
