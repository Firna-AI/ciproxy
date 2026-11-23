# Overflow API

Set `ApiState.overflow` to the runtime's shared `Arc<dyn Overflow>`. Use
`None` when the deployment has no configured provider.

GET `/workspaces/{workspace_id}/overflow` serves Admins in Setup and
Settings. The `/overflow/summary` route serves every current workspace role.
Both return the workspace switch, the wait, the next UTC month, and one
entry for each supported provider. Each entry gives its deployment
availability, whether billing account selection is needed, and whether the
workspace has offered the provider in label orders.

PUT `/workspaces/{workspace_id}/overflow` changes the workspace switch and
wait together. Each connection and limit route takes one provider in
`/workspaces/{workspace_id}/overflow/providers/{provider}`. The
[OpenAPI document](../../ts/api/generated/openapi.json) gives the methods and
request fields.

Only an Admin can check a key or make a change. A key check saves nothing.
Connect and Change key both verify the key and save a positive monthly spend
limit for that provider. A refused key leaves the saved key in place. The
HTTP 422 problem has code `provider_key_rejected` and names the provider.

The Boat key check returns the accessible wallets and the likely wallet.
A personal wallet uses the identifier `personal`. A Boat organization
uses its own ID. The selected wallet follows Boat's active choice, or the
first wallet when none is active. The connection request sends the chosen
wallet. A request with no wallet uses `personal`.

The provider entry holds the account name and the connection and
rejection dates. A deployment without that provider's configuration still
serves its saved account facts and permits Disconnect. Disconnect keeps
the spend limit and the offered mark. Key replacement and Disconnect let
running jobs finish with their saved keys. Response bodies contain no
key or ciphertext.

Spend reports confirmed costs, unconfirmed costs, and the part of the
unconfirmed costs whose bounds have settled. Each display amount rounds
once to the nearest cent, with half cents rounded away from zero. The
limit check uses exact costs. An unbounded start blocks admission and
makes the exact limit comparison unavailable.

[Overflow](../../docs/protocol/overflow.md) defines availability and the
connection rules. [Spend limit](../../docs/protocol/overflow-spend.md)
defines budget admission.
