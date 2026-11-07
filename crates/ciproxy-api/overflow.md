# Boat overflow API

Set `ApiState.overflow` to the runtime's shared `Arc<dyn Overflow>`. Use
`None` when the deployment has no shared Boat snapshot.

- GET `/workspaces/{workspace_id}/overflow` gives an Admin the data of the
  Boat step of Setup and of Settings under Overflow.
- GET `/workspaces/{workspace_id}/overflow/summary` gives the same data to
  each current workspace role, for the Machines page.
- POST `/workspaces/{workspace_id}/overflow/connection` connects a key or
  replaces it. It needs `api_key` and a `spend_limit_cents` above zero.
  `wallet` is `personal` by default.
- DELETE `/workspaces/{workspace_id}/overflow/connection` disconnects Boat.
- PUT `/workspaces/{workspace_id}/overflow` saves the complete policy:
  `enabled`, `wait_seconds`, and a `spend_limit_cents` above zero.

Only an Admin can change Boat data. The waits are 0, 15, 30, 60, and 120
seconds. Boat checks each key before ciproxy saves it. A key that Boat
refuses returns HTTP 422 with `boat_key_rejected`, and the saved key stays.

Key replacement and disconnect let running Boat jobs finish. Disconnect
keeps the saved limit for the next connection. Setup skips Boat when it
sends no connection request.

Responses hold no key or ciphertext. `account` gives the Boat account name,
`connected_at`, and `rejected_at`. `available` is false when the deployment
has no Boat service. Connection changes then return HTTP 503.
`next_month_at` is the first instant of the next UTC month.

`spend` is null when Boat is off or its refresh fails. `confirmed_cents`
holds the provider-confirmed costs. `unconfirmed_cents` holds the pending
bounds and the settled bounds, and `settled_bound_cents` gives the settled
part. Each amount rounds once to the nearest cent, with half-cents away
from zero. Spend admission still uses the exact amounts.

`bounded` is false when a start has no safe cost bound. `limit_reached`
compares the exact counted spend with the saved limit.
