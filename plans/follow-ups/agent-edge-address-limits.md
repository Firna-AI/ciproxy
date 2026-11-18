# One address can hold every gateway handshake

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-17, from `conductor/final-review-r2-runtime`, option B
  of review finding 1302.
- **Location:** `deploy/terraform/modules/edge/backends.tf:82-106`, the
  backend service `google_compute_backend_service.runtime`, which serves
  the agent route `/agent/`.
- **Reason:** The user asked for per-address limits at the load balancer
  edge as separate infrastructure work, in addition to the gateway limits.

- **Terms:**
  - **Gateway:** the machine gateway of the control plane. Each agent opens
    a WebSocket connection to it at the route `/agent/`.
  - **Upgrade:** the HTTPS request that opens the WebSocket connection of an
    agent.
  - **Handshake:** the start of a connection. It lasts from the upgrade
    until the gateway accepts the install token or the machine credential,
    or until the connection ends.
  - **Edge:** the global external HTTPS load balancer of an environment. It
    sends the requests for `/agent/` to the control plane.
  - **Security policy:** a Google Cloud Armor policy on a backend service.
    Its rules can limit the request rate of each client address.
- **Background:**
  - On 2026-11-17, the user approved limits in the gateway for connections
    before authentication. "Connection limits" in
    `docs/protocol/agent-connection.md` states them.
  - The gateway waits at most 15 seconds for the first complete message.
    Then it closes the connection with the WebSocket code 1008 and the
    reason `[gateway] handshake timed out`.
  - Each close waits at most five seconds for the reply of the peer. A
    handshake keeps its place until its close ends. So one idle connection
    holds a place for up to about 20 seconds.
  - Each gateway instance has at most eight handshakes in progress. It
    answers each further upgrade with the HTTP status 503.
  - The limit counts all client addresses together. One source that opens
    eight idle connections about every 20 seconds can keep all places busy.
  - The production deployment has one control-plane pod with a 1 GiB
    memory limit. So the gateway has one instance and eight places in total.
  - The backend service `runtime` in
    `deploy/terraform/modules/edge/backends.tf` has no security policy.
  - On the same day, the user asked for per-address limits at the edge, for
    example a security policy, in addition to the gateway limits. Edge
    limits make it harder for one source to take all places. They do not
    replace the gateway limits.
- **Task:**
  - Choose the edge policy for upgrades at `/agent/`: the rule, its key,
    its threshold, its interval, and its action.
  - Match only upgrades at `/agent/`. Other HTTPS routes stay outside this
    task.
  - Let normal reconnects through. After a restart of the control plane,
    every agent connects again. Many machines of one customer can share one
    address.
  - Before you apply the policy, get the user's approval of it. It adds a
    new rejection at the edge.
  - Add the policy in Terraform. Attach it to the backend service in
    `deploy/terraform/modules/edge/backends.tf`.
  - In the same change, add the per-address limit to "Connection limits"
    in `docs/protocol/agent-connection.md`.
  - Check the rule in preview before production.
- **Impact if no action:** One source can keep all eight handshake places
  busy. Each agent that connects again then gets 503 until a place is free.
  The dashboard shows a machine without contact for 60 seconds as offline,
  and ciproxy starts no runner on an offline machine.
- **Needs:** This change deployed, rights to apply the edge Terraform in
  preview and production, and the user's approval of the policy.
- **Done when:** In preview and production, upgrades at `/agent/` from one
  address over the chosen limit get the edge rejection. Upgrades from other
  addresses still reach the gateway. The protocol doc states the limit.
