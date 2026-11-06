# Agent updates and releases

The daemon checks for a new agent version every hour with `cli-updater`.
The default release server is `https://get.ciproxy.dev`.
`CIPROXY_RELEASE_SERVER_URL` can select a staging HTTPS release server.
The update requires a checksum and restarts only the agent unit.
A failed update cycle keeps the next hourly check.
The updater source is pinned in the workspace manifest.
`RUST_LOG` selects logging verbosity. JSON operation events remain available.
An outer safety filter always excludes `tungstenite` and `tokio_tungstenite`.
This filter also checks normalized targets from the log-to-tracing bridge.
User directives cannot enable transport payload diagnostics.

The systemd service reads `/etc/ciproxy-agent/environment` when present.
Keep this root-only settings file at mode `0600`.

Release-plz tags only the agent, with `ciproxy-agent-vVERSION`.
The release workflow runs the complete gate in a read-only PostgreSQL job.
Versioning requires that job to pass.
Package builds run read-only. A separate job uploads the verified assets.
Action commits and release tool versions are pinned.
Set `RELEASE_PLZ_TOKEN` to a repository-scoped token with release and pull
request write access. Its tags must trigger the archive workflow.
The release server needs access to the private release assets.
It uses binary `ciproxy-agent` and tag prefix `ciproxy-agent-v`.

[Agent guide](README.md) gives installation, commands, and code entry points.
