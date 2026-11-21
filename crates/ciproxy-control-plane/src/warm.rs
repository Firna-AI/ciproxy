//! The documented 24-hour proportional warm-runner allocation.

use ciproxy_protocol::identifiers::OrganizationId;

pub(crate) fn allocate(count: usize, history: &[(OrganizationId, u64)]) -> Vec<OrganizationId> {
    let total: u128 = history.iter().map(|(_, jobs)| u128::from(*jobs)).sum();
    if history.is_empty() {
        return Vec::new();
    }
    let mut shares: Vec<usize> = history
        .iter()
        .map(|(_, jobs)| {
            (count as u128 * u128::from(*jobs))
                .checked_div(total)
                .unwrap_or(0) as usize
        })
        .collect();
    let mut output = Vec::with_capacity(count);
    for (index, (organization, _)) in history.iter().enumerate() {
        output.extend(std::iter::repeat_n(organization.clone(), shares[index]));
    }
    while output.len() < count {
        let index = if total == 0 {
            output.len() % history.len()
        } else {
            (0..history.len())
                .min_by_key(|index| (std::cmp::Reverse(history[*index].1), shares[*index], *index))
                .unwrap_or(0)
        };
        shares[index] += 1;
        output.push(history[index].0.clone());
    }
    output
}

#[cfg(test)]
#[path = "_tests_/warm_tests.rs"]
mod warm_tests;
