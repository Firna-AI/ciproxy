//! Slot-density upper bounds safely leave only unusable root disk as spare.

use super::spare;

#[test]
fn spare_excludes_every_feasible_root_combination_without_knapsack_search() {
    assert_eq!(spare(6, 645, &[(1, 80), (2, 130)], &[]), 165);
    assert_eq!(spare(6, 645, &[(1, 80), (2, 260)], &[]), 0);
    assert_eq!(spare(3, 82, &[(1, 80)], &[]), 0);
    assert_eq!(spare(6, 645, &[(1, 80), (7, 600), (1, 700)], &[]), 165);
    assert_eq!(spare(3, 279, &[(1, 80)], &[(1, 100), (1, 100)]), 0);
    assert_eq!(spare(3, 279, &[(1, 80)], &[(1, 100)]), 19);
    assert_eq!(spare(3, 279, &[(1, 80)], &[(1, 80), (1, 80)]), 39);
}
