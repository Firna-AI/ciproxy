# No test reads a second page of groups

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from pull request #142
  (`conductor/ciproxy-runner-group-per-environment`), the check of review
  finding 6.
- **Location:** `crates/ciproxy-github/src/rest/groups.rs:57`,
  `crates/ciproxy-github/tests/integration/resource_http.rs`,
  `crates/ciproxy-github/tests/integration/fake_http.rs`.
- **Reason:** The pull request that found it did not change the paging
  code, so its review left the gap for a separate change.

- **Terms:**
  - **Managed runner group:** the private GitHub runner group that one
    environment keeps in each organization. Its name is
    `ciproxy-<app id>-<organization id>`.
- **Background:** When no saved group ID finds the managed group,
  `RestClient::sync_group` in `crates/ciproxy-github/src/rest/groups.rs`
  lists the runner groups of the organization with `per_page=100`. It
  reads page after page until it finds the full managed name, or until a
  page holds fewer than 100 groups. It then updates the found group, or
  creates one. The fake GitHub server of the integration tests in
  `crates/ciproxy-github/tests/integration/` answers every list request
  with one short page.
- **Problem:** No test lists a second page of runner groups. A change
  that reads only the first page passes every test.
- **Impact if no action:** In an organization with more than 100 runner
  groups, such a change misses a managed group on a later page. The sync
  then tries to create a second group with the same name, so the
  organization loses its saved group or its sync fails.
- **Root question:** Which test fails when the lookup by name stops
  before the page that holds the managed group?
- **Options:**
  - **A.** Let the fake server answer a list request with a full first
    page of 100 other groups and a second page that holds the managed
    group. Add one case to the group tests of
    `crates/ciproxy-github/tests/integration/recovery_http_tests.rs` with
    no saved ID. Assert the found group ID, zero creates, and writes only
    to that group. Effort: Small.
  - **B.** Make the fake server keep its groups by ID and page them by
    the `page` and `per_page` query, for every group test. Effort:
    Medium.
- **Left out:** A unit test of the private paging loop. The `testing`
  skill says to test through the public entry point.
- **Wider fix:** B. One stateful fake pages every list in the same way.
- **Previous examples:** Review finding 6 of #142 found the same kind of
  gap: the fake list held one group, so a lookup that read only the first
  group passed.
- **Recommendation:** A. It fails on the named break for Small effort.
- **Action:** fix after merge, apply A, because the pull request that
  found the gap does not change the paging code.
