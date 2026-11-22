//! A saved state restores only for the VM that it was saved from.

use std::path::PathBuf;

use super::seats::{
    MAX_AGE, SAVED_STATE_FORMAT, SavedState, SeatNeeds, StartKind, free_seat, needs_build,
    start_kind,
};
use super::vm_control::StopReason;
use super::vm_spec::{ExitReason, SpecError, VM_SPEC_FORMAT, VmExit, VmSpec};

const SAVED_AT: u64 = 1_800_000_000;

fn saved() -> SavedState {
    SavedState {
        format: SAVED_STATE_FORMAT,
        seat: 1,
        image: "macos-26.7.2-1".to_owned(),
        mac_address: "02:ab:cd:00:00:01".to_owned(),
        agent_version: "0.5.0".to_owned(),
        saved_at_unix_seconds: SAVED_AT,
    }
}

fn needs(now: u64) -> SeatNeeds<'static> {
    SeatNeeds {
        seat: 1,
        newest_image: "macos-26.7.2-1",
        mac_address: "02:ab:cd:00:00:01",
        agent_version: "0.5.0",
        now_unix_seconds: now,
    }
}

#[test]
fn the_lowest_free_seat_wins_and_two_is_the_limit() {
    assert_eq!(free_seat(&[]), Some(0));
    assert_eq!(free_seat(&[0]), Some(1));
    assert_eq!(free_seat(&[1]), Some(0));
    assert_eq!(free_seat(&[1, 0]), None);
}

#[test]
fn a_fresh_matching_state_restores() {
    assert_eq!(
        start_kind(Some(&saved()), needs(SAVED_AT + 60)),
        StartKind::Restore
    );
    assert_eq!(start_kind(None, needs(SAVED_AT + 60)), StartKind::Cold);
}

#[test]
fn any_mismatch_or_age_boots_cold() {
    let limit = MAX_AGE.as_secs();
    let mut cases = vec![
        (saved(), needs(SAVED_AT + limit)),
        (saved(), needs(SAVED_AT - 1)),
    ];
    let mut other = needs(SAVED_AT);
    other.seat = 0;
    cases.push((saved(), other));
    for change in [
        |state: &mut SavedState| state.image = "macos-26.7.2-2".to_owned(),
        |state: &mut SavedState| state.mac_address = "02:ab:cd:00:00:02".to_owned(),
        |state: &mut SavedState| state.agent_version = "0.5.1".to_owned(),
        |state: &mut SavedState| state.format = 2,
    ] {
        let mut state = saved();
        change(&mut state);
        cases.push((state, needs(SAVED_AT)));
    }
    for (state, needs) in cases {
        assert_eq!(
            start_kind(Some(&state), needs),
            StartKind::Cold,
            "{state:?}"
        );
    }
}

#[test]
fn a_build_runs_only_on_a_free_seat_of_an_accepting_machine() {
    let stale = needs(SAVED_AT + MAX_AGE.as_secs());
    assert!(needs_build(Some(&saved()), stale, true, true));
    assert!(needs_build(None, needs(SAVED_AT), true, true));
    assert!(!needs_build(Some(&saved()), needs(SAVED_AT), true, true));
    assert!(!needs_build(None, needs(SAVED_AT), false, true));
    assert!(!needs_build(None, needs(SAVED_AT), true, false));
}

#[test]
fn saved_state_files_refuse_unknown_fields() {
    let text = serde_json::to_string(&saved()).expect("encode");
    assert_eq!(
        serde_json::from_str::<SavedState>(&text).expect("decode"),
        saved()
    );
    let extra = text.replacen('{', "{\"token\":\"x\",", 1);
    assert!(serde_json::from_str::<SavedState>(&extra).is_err());
}

fn spec() -> VmSpec {
    VmSpec {
        format: VM_SPEC_FORMAT,
        seat: 0,
        cpus: 4,
        memory_bytes: 8 << 30,
        bundle: PathBuf::from("/var/db/ciproxy/area/vms/x/bundle"),
        start: StartKind::Cold,
        saved_state: None,
    }
}

#[test]
fn the_helper_refuses_an_inconsistent_spec() {
    assert_eq!(spec().check(), Ok(()));
    let restore = VmSpec {
        start: StartKind::Restore,
        saved_state: Some(PathBuf::from("/var/db/ciproxy/area/seats/0/current/state")),
        ..spec()
    };
    assert_eq!(restore.check(), Ok(()));
    let cases = [
        (
            VmSpec {
                format: 2,
                ..spec()
            },
            SpecError::Format,
        ),
        (VmSpec { seat: 2, ..spec() }, SpecError::Seat),
        (VmSpec { cpus: 0, ..spec() }, SpecError::Size),
        (
            VmSpec {
                memory_bytes: 0,
                ..spec()
            },
            SpecError::Size,
        ),
        (
            VmSpec {
                start: StartKind::Restore,
                ..spec()
            },
            SpecError::SavedState,
        ),
        (
            VmSpec {
                saved_state: restore.saved_state.clone(),
                ..spec()
            },
            SpecError::SavedState,
        ),
    ];
    for (spec, error) in cases {
        assert_eq!(spec.check(), Err(error), "{spec:?}");
    }
}

#[test]
fn the_exit_file_uses_the_control_words() {
    let exit = VmExit {
        reason: ExitReason::from(StopReason::GuestShutdown),
    };
    let text = serde_json::to_string(&exit).expect("encode");
    assert_eq!(
        text,
        format!("{{\"reason\":\"{}\"}}", StopReason::GuestShutdown.word())
    );
    assert_eq!(serde_json::from_str::<VmExit>(&text).expect("decode"), exit);
}
