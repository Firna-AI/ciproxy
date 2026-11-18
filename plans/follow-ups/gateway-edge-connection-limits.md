# Agent route lacks limits for each client address

- **Severity:** High. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`: option B of finding 1302,
  which the user made a production launch step on 17 November 2026.
- **Source:** Review.
- **Category:** Security.
- **Location:** `deploy/terraform/modules/edge/backends.tf`,
  `google_compute_backend_service.runtime`, the tests under
  `deploy/terraform/edge/`, `crates/ciproxy-gateway/src/transport.rs`,
  `authenticate` and the agent route of `router`, and
  `docs/protocol/agent-connection.md`, "Connection limits".
- **Reason:** B needs a security policy at the load balancer, which is
  infrastructure, and the user made it a step of the production launch.

- **Terms:**
  - **Agent route:** the WebSocket route `/agent/` of the control plane.
    Each machine connects there.
  - **Handshake:** the time from the WebSocket upgrade until the gateway
    accepts the credential of the first message, `Hello` or `Register`, or
    until the connection ends. A failed handshake ends after its close.
  - **Edge:** the external Application Load Balancer of Google Cloud in
    front of the control plane, from `deploy/terraform/modules/edge/`.
  - **Security policy:** a Google Cloud Armor policy on a backend service.
    It can throttle or block the requests of each client address.
- **Background:**
  - Each machine opens a WebSocket to `/agent/` and sends `Hello` or
    `Register` first. `authenticate` in
    `crates/ciproxy-gateway/src/transport.rs` reads that whole message
    before it checks any credential.
  - The control plane runs as one pod with a memory limit of 1 GiB.
  - The backend service `runtime` in
    `deploy/terraform/modules/edge/backends.tf` keeps a connection open for
    up to 86,400 seconds. Its `max_rate_per_endpoint = 100` sets a
    balancing target for each endpoint. It sets no limit for each client
    address, and the Terraform has no security policy.
  - Before the final review, the gateway accepted a first message of up to
    64 MiB before authentication. It waited for that message with no time
    limit, and it limited neither the open connections nor the pending
    handshakes.
  - On 17 November 2026 the final review chose option A for the gateway,
    as a fix in the branch. It gives the first message a deadline, lowers
    the message limit of the agent route, and caps the handshakes in
    progress.
  - These limits bound the memory of the pod. They do not tell one client
    address from another.
  - On 18 November 2026, commit `512224f` applied A with these defaults.
    "Connection limits" in `docs/protocol/agent-connection.md` states them.
    - The first complete message must arrive within 15 seconds after the
      upgrade. Otherwise the gateway closes the connection with the code
      1008 and the reason `[gateway] handshake timed out`.
    - Each gateway instance has at most eight handshakes in progress. The
      cap counts all client addresses together.
    - The gateway answers each upgrade above the cap with the HTTP status
      503. The agent can try again later.
    - Each frame and each reassembled message holds at most 16 MiB. This
      also holds for full reports, because a report has no finite largest
      valid size.
    - Each close sends its close frame and waits for the reply of the peer.
      The send and the wait take at most five seconds in total.
  - A failed handshake keeps its place in the cap during its close. So a
    silent client holds a place for about 20 seconds: 15 seconds for the
    first message, and up to five seconds for the close.
  - The one pod gives one gateway instance. So production has eight
    handshake places in total.
  - A machine without contact for 60 seconds shows as offline, and no new
    runner starts on it.
  - Normal reconnects come in bursts. After a deploy or a restart of the
    control plane, every machine connects again. Many machines of one
    customer can share one NAT address. A limit for each address must let
    such a burst through.
  - Route match:
    - `google_compute_backend_service.runtime` creates one backend service
      for each runtime, `release` and `control_plane`. The agent route uses
      `control_plane`.
    - A security policy applies to the whole backend service. The
      `control_plane` backend service serves the agent route and other
      routes of the control plane.
    - Each rule of the policy chooses the requests that it counts.
    - The wider fix of B covers every route behind the same backend
      service. A rule that counts only upgrades at `/agent/` is narrower
      than that recorded choice.
    - A rule that counts every route must also let through the normal
      request rate of those other routes from one address.
  - A limit for each client address is a new requirement of the agent
    route. Add it to "Connection limits" in
    `docs/protocol/agent-connection.md` in the same change.
- **Problem:** After the gateway limits, one client address can still open
  connections up to the handshake cap, and open new ones as each deadline
  ends. About eight silent connections every 20 seconds keep the cap full.
  Machines then find the cap full and cannot connect.
- **Impact if no action:** An unauthenticated client can keep machines from
  connecting or reconnecting, for example after a deploy. Their jobs then
  do not start on own machines.
- **Root question:** How does the service stop one unauthenticated client
  from using the whole connection budget of the agent route?
- **Options:**
  - **A.** The gateway limits that the final review applies in the branch.
    - A deadline for the first message, such as 15 seconds.
    - A message limit near the size of the largest valid report.
    - A cap on the handshakes in progress.
    - Each limit has a regression test.
    - Alone, A fails this goal, because one address can fill the cap.
    - Effort: Small.
  - **B.** A plus a limit for each client address at the edge.
    - Add a Cloud Armor security policy to the backend service of the agent
      route, with a rate-based rule for each client address. The rule
      throttles or blocks an address above a number of new connections in
      a time window.
    - Add the policy to the Terraform edge module, with its test.
    - Prove it once on the main preview: one address above the limit gets
      refused, and a machine still connects.
    - Choose the limit from the reconnect rate of the largest customer
      behind one NAT address.
    - Cost: new infrastructure and a live proof.
    - Effort: Medium.
  - **C.** A plus a handshake cap for each client address in the gateway.
    - The gateway takes the client address from the `X-Forwarded-For`
      header that the load balancer sets.
    - Cost: the gateway must trust only the entry that the load balancer
      adds. The flood still reaches the one pod. Many machines of one
      customer behind one NAT address share one cap.
    - Effort: Small.
- **Left out:**
  - A check of the credential in the HTTP upgrade request: it changes the
    open agent protocol and needs an agent release. The upgrade requests
    still need limits, which B and C give with no protocol change.
  - A larger handshake cap: it fails the goal, because one address can
    still fill any fixed cap.
- **Wider fix:** B. It limits each address before the traffic reaches the
  pod, for every route behind the same backend service.
- **Previous examples:** Round 2 of the same review found that the queue of
  agent events counted messages but did not bound their memory. The final
  review fixed that queue in the branch.
- **Related:** `plans/follow-ups/production-launch.md`: the production
  launch. The user made this item a step of the launch.
- **Recommendation:** B. It stops the flood at the edge, before it reaches
  the one pod, and it needs no trust in a forwarded header. C is cheaper,
  but the traffic still reaches the pod.
- **Action:** fix after merge, apply B, because it needs a Cloud Armor
  policy at the load balancer. The user made it a production launch step
  on 17 November 2026.
