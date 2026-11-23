# The trace test cannot see reqwest log records

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch
  `calummoore/h3-macos-release-from-r2-qjch`, review round 2, finding 12.
- **Location:** `crates/ciproxy-control-plane/tests/signed_image_tests.rs`,
  `signed_manifest_and_bundle_metadata_reads_keep_credentials_out_of_trace_logs`;
  `crates/ciproxy-control-plane/src/cli.rs`, the subscriber `.init()`;
  `crates/ciproxy-gateway/src/logging.rs`, the mandatory guard;
  `crates/ciproxy-control-plane/tests/runtime/takeout_installation_http.rs`,
  its loopback client.
- **Reason:** The log setup of the trace test and the hosted log guard are
  older than the branch that found this.

- **Terms:**
  - **Log bridge:** the `LogTracer` that `.init()` of `tracing-subscriber`
    installs, so that records of the `log` crate reach the subscriber.
  - **Byte log:** reqwest's `connection_verbose` records, under the
    target `reqwest::connect::verbose`, which hold the request line with
    its signed URL.
- **Background:** Signed release reads carry their credential in the URL.
  Production installs the hosted subscriber with `.init()` in `cli.rs`.
  The trace test installs it with `set_global_default`, so it has no log
  bridge. Its only probe is a native `tracing` event. The test now uses
  the production client policy, `release_client` in
  `crates/ciproxy-control-plane/src/image_release.rs`, with no proxy. The
  takeout test client in `takeout_installation_http.rs` still reads host
  proxy settings.
- **Problem:** A change that turns on `connection_verbose` in the release
  client would write signed URLs to trace logs, and the trace test would
  still pass, because reqwest writes those records only through `log`.
- **Impact if no action:** A logging change can expose a signed release
  URL at trace level, while the tests stay green.
- **Root question:** How does a test see every record that production logs
  for a signed release read, and fail when a record holds the signature?
- **Options:**
  - **A.** Install the subscriber with `.init()` in the trace test, as
    `cli.rs` does, add a `log::trace!` probe, and add an R2 case. Effort:
    Small.
  - **B.** Add `reqwest::connect::verbose` to the mandatory guard in
    `crates/ciproxy-gateway/src/logging.rs`, beside the tungstenite
    targets, and test it with a `connection_verbose(true)` client. Effort:
    Small.
  - **C.** A and B, and move the takeout test client to a client with no
    proxy. Effort: Medium.
- **Left out:** A gate that refuses `connection_verbose`: B stops the same
  records for every hosted client without a new gate.
- **Wider fix:** C. The test then shares the client and the log setup with
  production, and the guard drops the byte log for every hosted client.
- **Previous examples:** Commit `703b9b19` added the trace test with its
  own client.
- **Recommendation:** C.
- **Action:** fix after merge, apply C, because it changes the hosted log
  policy and test code that are older than this branch.
