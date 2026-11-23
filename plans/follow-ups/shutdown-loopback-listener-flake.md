# Shutdown test saw three loopback listeners once

- **Severity:** Medium. **Effort:** Medium. **Confidence:** Low.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch
  `calummoore/ciproxy-providers-interface-s8rp`, the dashboard API work.
- **Location:** `crates/ciproxy-control-plane/tests/startup_shutdown/sockets.rs:73`.
- **Reason:** The test failed once in a full run and passed alone and in
  the next full run, so the cause needs its own investigation.

- **Source:** Implementation.
- **Category:** Test.
- **Background:**
  - The test `sigterm_stops_new_connections_and_drains_an_active_http_request`
    starts the control-plane executable. It finds the one loopback port
    that the process listens on, then sends SIGTERM during a request.
  - On 23 November 2026, one full `cargo test --workspace` run failed it
    with this text: `the executable must own exactly one loopback
    listener: [127.0.0.1:43719, 127.0.0.1:43889, 127.0.0.1:33675]`.
  - The test passed alone, and the next full run passed.
  - The change in that run renamed dashboard API types. It did not touch
    the startup, shutdown, or listener code.
- **Problem:** The test reads the listeners of the process through procfs.
  Under parallel load, it can see other listeners, such as those of a
  sibling test process, or listeners that the process opens for a short
  time. The suspected source is the listener discovery, not the product.
- **Impact if no action:** The gate can fail now and then with no product
  fault, and a real shutdown fault can hide behind it.
- **Root question:** How does the test learn the one port that the
  executable serves?
- **Options:**
  - **A.** Make the executable print its bound address on start, and read
    it in the test instead of scanning procfs. Effort: Medium.
  - **B.** Filter the procfs scan to the inodes of the exact child process
    and its open sockets only. Effort: Medium.
- **Left out:** A retry or a longer timeout, because the `testing` skill
  forbids both.
- **Wider fix:** A. A reported address removes the discovery step for every
  test that starts the executable.
- **Previous examples:** None found.
- **Recommendation:** A. The test then reads a fact that the process
  states, not a guess from the system.
- **Action:** fix after merge, apply A, because the failure is rare and the
  change touches the executable's startup output.
