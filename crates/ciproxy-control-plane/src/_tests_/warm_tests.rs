//! Hand-worked allocations from the shared 24-hour policy.

use ciproxy_protocol::identifiers::OrganizationId;

use super::allocate;

#[test]
fn warm_leftovers_follow_recent_demand_before_equalizing_allocations() {
    let first = OrganizationId("first".to_owned());
    let second = OrganizationId("second".to_owned());
    assert_eq!(
        allocate(2, &[(first.clone(), 9), (second.clone(), 1)]),
        vec![first.clone(), first.clone()]
    );
    assert_eq!(
        allocate(3, &[(first.clone(), 4), (second.clone(), 4)]),
        vec![first.clone(), second.clone(), first]
    );
}

#[test]
fn no_history_allocates_in_stable_round_robin_order() {
    let first = OrganizationId("first".to_owned());
    let second = OrganizationId("second".to_owned());
    assert_eq!(
        allocate(5, &[(first.clone(), 0), (second.clone(), 0)]),
        vec![first.clone(), second.clone(), first.clone(), second, first]
    );
    assert_eq!(allocate(2, &[]), Vec::<OrganizationId>::new());
}
