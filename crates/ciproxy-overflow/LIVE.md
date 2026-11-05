# Live Boat and Cloud KMS probes

These commands contact Boat or Cloud KMS and can incur charges.
Credential arguments are file paths.
Create those files outside Git with mode 0600.
The scripts read paths from environment variables and never print contents.
`--help` runs without credentials or network access.

```sh
export BOAT_API_KEY_FILE=/private/boat-key
export GCP_ACCESS_TOKEN_FILE=/private/gcp-oauth-token
export KMS_KEY_RESOURCE=projects/P/locations/L/keyRings/R/cryptoKeys/K

crates/ciproxy-overflow/scripts/kms-roundtrip.sh
crates/ciproxy-overflow/scripts/boat-live.sh account

mkdir -p .context/ciproxy-mvp/milestone-12/live
crates/ciproxy-overflow/scripts/boat-live.sh create \
  --state-file .context/ciproxy-mvp/milestone-12/live/boat.json \
  --snapshot SHARED_SNAPSHOT_OR_SHARE_CODE --size large
crates/ciproxy-overflow/scripts/boat-live.sh recover \
  --state-file .context/ciproxy-mvp/milestone-12/live/boat.json
crates/ciproxy-overflow/scripts/boat-live.sh inspect \
  --state-file .context/ciproxy-mvp/milestone-12/live/boat.json
crates/ciproxy-overflow/scripts/boat-live.sh runner \
  --state-file .context/ciproxy-mvp/milestone-12/live/boat.json \
  --registration-file /private/boat-registration.json
crates/ciproxy-overflow/scripts/boat-live.sh usage \
  --state-file .context/ciproxy-mvp/milestone-12/live/boat.json
crates/ciproxy-overflow/scripts/boat-live.sh delete \
  --state-file .context/ciproxy-mvp/milestone-12/live/boat.json
```

Use `--trial` with `create` on a free trial.
The private registration file uses the
[image's Boat format](../../image/README.md#boat-small-and-default-smoke).
It needs a live JIT registration and an HTTPS callback URL and token.
The runner probe adjusts the internal runner ID to the recorded delivery.
The live state file contains identities and phases, without credentials.
An existing create state always selects recovery reads.
Run `delete` even when a later probe fails.
Use a separate state file for each sandbox.
These probes do not supply the control-plane callback server or a GitHub
registration issuer.
The control-plane runtime owns those integrations.

Return to the [crate guide](README.md#live-commands).
