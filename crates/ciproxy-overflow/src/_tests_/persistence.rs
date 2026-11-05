//! Unimock persistence answers retain captured service writes for assertions.

use std::sync::{Arc, Mutex};

use ciproxy_store_interface::overflow::store::{
    OverflowSession, OverflowSessionMock, OverflowStore, OverflowStoreMock,
};
use unimock::{MockFn, Unimock, matching};

use super::fixtures::State;

pub(super) fn store(state: Arc<Mutex<State>>, calls: &[&str]) -> Arc<dyn OverflowStore> {
    let read = state.clone();
    let save = state.clone();
    let account = state.clone();
    let reserve = state.clone();
    let launch = state.clone();
    let refusal = state.clone();
    let accept = state.clone();
    let connect = state.clone();
    let snapshot = OverflowSessionMock::snapshot
        .some_call(matching!())
        .answers_arc(Arc::new(move |_| {
            Ok(read.lock().unwrap().snapshot.clone().unwrap())
        }))
        .at_least_times(1);
    let save_start = OverflowSessionMock::save_start
        .some_call(matching!(_))
        .answers_arc(Arc::new(move |_, record| {
            let mut state = save.lock().unwrap();
            state.phases.push(record.phase);
            let starts = &mut state.snapshot.as_mut().unwrap().starts;
            if let Some(existing) = starts
                .iter_mut()
                .find(|start| start.delivery_id == record.delivery_id)
            {
                *existing = record.clone();
            } else {
                starts.push(record.clone());
            }
            Ok(())
        }))
        .at_least_times(1);
    let save_account = OverflowSessionMock::save_account
        .some_call(matching!(_))
        .answers_arc(Arc::new(move |_, record| {
            let mut state = account.lock().unwrap();
            *state
                .snapshot
                .as_mut()
                .unwrap()
                .accounts
                .iter_mut()
                .find(|account| account.account.id == record.account.id)
                .unwrap() = record.clone();
            Ok(())
        }))
        .at_least_times(1);
    let reserve_start = OverflowSessionMock::reserve_start
        .some_call(matching!(_, _, _))
        .answers_arc(Arc::new(move |_, record, ciphertext, hash| {
            let mut state = reserve.lock().unwrap();
            state.snapshot.as_mut().unwrap().starts.push(record.clone());
            state.phases.push(record.phase);
            state
                .launches
                .insert(record.delivery_id, ciphertext.clone());
            state.hashes.insert(record.delivery_id, hash.clone());
            Ok(true)
        }))
        .at_least_times(1);
    let encrypted_launch = OverflowSessionMock::encrypted_launch
        .some_call(matching!(_))
        .answers_arc(Arc::new(move |_, delivery| {
            Ok(launch.lock().unwrap().launches.get(&delivery).cloned())
        }))
        .at_least_times(1);
    let save_refusal = OverflowSessionMock::save_refusal
        .some_call(matching!(_, _))
        .answers_arc(Arc::new(move |_, account, record| {
            let mut state = refusal.lock().unwrap();
            let snapshot = state.snapshot.as_mut().unwrap();
            snapshot.accounts[0] = account.clone();
            *snapshot
                .starts
                .iter_mut()
                .find(|start| start.delivery_id == record.delivery_id)
                .unwrap() = record.clone();
            state.phases.push(record.phase);
            Ok(())
        }))
        .at_least_times(1);
    let accept_end = OverflowSessionMock::accept_end
        .some_call(matching!(_, _, _))
        .answers_arc(Arc::new(move |_, _, _, _| {
            Ok(accept
                .lock()
                .unwrap()
                .end_claims
                .pop_front()
                .expect("explicit claim response"))
        }))
        .at_least_times(1);
    let collect = OverflowSessionMock::collect
        .some_call(matching!(_))
        .answers(&|_, _| Ok(()))
        .at_least_times(1);
    let connect = OverflowSessionMock::connect
        .some_call(matching!(_, _))
        .answers_arc(Arc::new(move |_, record, limit| {
            let mut state = connect.lock().unwrap();
            let snapshot = state.snapshot.as_mut().unwrap();
            snapshot.accounts.push(record.clone());
            snapshot.spend_limit_usd_micros = Some(limit);
            Ok(())
        }))
        .at_least_times(1);
    let methods: Vec<_> = [
        "snapshot",
        "save_start",
        "save_account",
        "reserve_start",
        "encrypted_launch",
        "save_refusal",
        "accept_end",
        "collect",
        "connect",
    ]
    .into_iter()
    .filter(|method| calls.contains(method))
    .collect();
    let session = Arc::new(match methods.as_slice() {
        [] => Unimock::new(()),
        ["collect", "connect"] => Unimock::new((collect, connect)),
        ["snapshot"] => Unimock::new(snapshot),
        ["snapshot", "save_account"] => Unimock::new((snapshot, save_account)),
        ["snapshot", "save_account", "reserve_start"] => {
            Unimock::new((snapshot, save_account, reserve_start))
        }
        ["snapshot", "save_account", "reserve_start", "save_refusal"] => {
            Unimock::new((snapshot, save_account, reserve_start, save_refusal))
        }
        ["snapshot", "save_start"] => Unimock::new((snapshot, save_start)),
        ["snapshot", "save_start", "collect"] => Unimock::new((snapshot, save_start, collect)),
        ["snapshot", "save_start", "encrypted_launch"] => {
            Unimock::new((snapshot, save_start, encrypted_launch))
        }
        [
            "snapshot",
            "save_start",
            "save_account",
            "reserve_start",
            "encrypted_launch",
        ] => Unimock::new((
            snapshot,
            save_start,
            save_account,
            reserve_start,
            encrypted_launch,
        )),
        [
            "snapshot",
            "save_start",
            "save_account",
            "reserve_start",
            "save_refusal",
        ] => Unimock::new((
            snapshot,
            save_start,
            save_account,
            reserve_start,
            save_refusal,
        )),
        [
            "snapshot",
            "save_start",
            "save_account",
            "reserve_start",
            "save_refusal",
            "accept_end",
            "collect",
        ] => Unimock::new((
            snapshot,
            save_start,
            save_account,
            reserve_start,
            save_refusal,
            accept_end,
            collect,
        )),
        [
            "snapshot",
            "save_start",
            "save_account",
            "reserve_start",
            "encrypted_launch",
            "accept_end",
            "collect",
        ] => Unimock::new((
            snapshot,
            save_start,
            save_account,
            reserve_start,
            encrypted_launch,
            accept_end,
            collect,
        )),
        _ => panic!("unknown persistence fixture methods"),
    });
    let waiting = session.clone();
    let try_lock_overflow = OverflowStoreMock::try_lock_overflow
        .some_call(matching!(_))
        .answers_arc(Arc::new(move |_, _| {
            Ok(Some(
                Box::new((*session).clone()) as Box<dyn OverflowSession>
            ))
        }))
        .at_least_times(1);
    let lock_overflow = OverflowStoreMock::lock_overflow
        .some_call(matching!(_))
        .answers_arc(Arc::new(move |_, _| {
            Ok(Box::new((*waiting).clone()) as Box<dyn OverflowSession>)
        }))
        .at_least_times(1);
    let overflow_token_workspace = OverflowStoreMock::overflow_token_workspace
        .some_call(matching!(_))
        .answers_arc(Arc::new(move |_, _| {
            Ok(state
                .lock()
                .unwrap()
                .token_lookups
                .pop_front()
                .expect("explicit token lookup"))
        }))
        .at_least_times(1);
    if calls.contains(&"lock_overflow") {
        Arc::new(Unimock::new((
            try_lock_overflow,
            lock_overflow,
            overflow_token_workspace,
        )))
    } else {
        Arc::new(Unimock::new(try_lock_overflow))
    }
}
