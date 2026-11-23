# Remove the old sandbox exit route

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-23, from `calummoore/ciproxy-providers-interface-s8rp`.
- **Location:** `crates/ciproxy-control-plane/src/overflow_callback.rs`.
- **Reason:** Unfinished deliveries can still carry the old callback URL.

- **Background:** A delivery is the saved request to start an overflow
  sandbox. Its encrypted launch input carries the exit URL and its
  one-use token. New deliveries get `/hooks/overflow/runner-exit`.
  Earlier deliveries can still use `/hooks/boat/runner-exit`. An unsent
  delivery can launch after the rollout and still use its saved URL.
- **Task:** Check each deployed environment after all control-plane replicas
  that write the old URL have stopped. Record a UTC cutoff that covers
  their last possible saved delivery time. Count open deliveries at or
  before that cutoff. For example, use this query in `psql` with
  `cutover_at` set to that UTC timestamp:

  ```sql
  SELECT count(*)
  FROM overflow_starts
  WHERE ends_at IS NULL
    AND COALESCE(
      (record_json::jsonb->>'starts_at')::timestamptz <= :'cutover_at'::timestamptz,
      true
    );
  ```

  When every count is zero, remove the old route. Move route tests that
  cover exit claims and deletion to the neutral path. Update the route
  descriptions in the control-plane README and `overflow.md`.
- **Impact if no action:** The service continues to accept the old path
  with the same sandbox-scoped token.
- **Needs:** The deployed rollout cutoff and read access to each deployed
  environment's control-plane database.
- **Done when:** All deliveries that can hold the old URL have ended.
  The neutral path saves exit reports and the old path returns HTTP 404.
