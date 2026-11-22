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

Release-plz processes the agent, cache sidecar, and shared protocol crate.
A change to any of these packages updates their shared workspace version.
All three compare changes with the last `ciproxy-agent-vVERSION` tag.
Only the agent creates a tag and a release.
The release workflow packages both executables under that agent tag.
The release workflow runs the complete gate in a read-only PostgreSQL job.
Versioning requires that job to pass.
Package builds run read-only. A separate job uploads the verified assets.
Each job that runs Cargo installs the Rust toolchain of
`rust-toolchain.toml` with the rustup commands of CI.
Action commits and release tool versions are pinned.
Set `RELEASE_PLZ_TOKEN` to a repository-scoped token with release and pull
request write access. Its tags must trigger the archive workflow.
The release server needs access to the private release assets.
It uses binary `ciproxy-agent` and tag prefix `ciproxy-agent-v`.

## Release targets

Each release has these archives, in the form of
[Install and archives](../../deploy/release-server.md#install-and-archives):

| Target | Binaries | Build |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `ciproxy-agent`, `ciproxy-cache-sidecar` | Linux, glibc 2.17 or later |
| `aarch64-apple-darwin` | `ciproxy-agent` | `macos-15` runner, macOS 14 or later |

The release stays a draft until each package job passes. So a Mac never sees
a release without its archive, unless signing is not set up.

## macOS signing

The `package-macos` job signs the agent with a Developer ID, the hardened
runtime, and the entitlement `com.apple.security.virtualization`. Then it
sends the binary to Apple notarization with `notarytool` and checks the
ticket. [macos-release.sh](scripts/macos-release.sh) does this work.

The job reads these inputs from the `macos-signing` GitHub environment or
from the repository:

| Name | Kind | Value |
| --- | --- | --- |
| `MACOS_SIGNING_CERTIFICATE_P12_BASE64` | Secret | Base64 of a `.p12` file with one Developer ID Application certificate and its key |
| `MACOS_SIGNING_CERTIFICATE_PASSWORD` | Secret | Password of that `.p12` file |
| `MACOS_NOTARY_API_KEY_P8_BASE64` | Secret | Base64 of an App Store Connect API key (`.p8`) |
| `MACOS_NOTARY_API_KEY_ID` | Secret | ID of that API key |
| `MACOS_NOTARY_API_ISSUER_ID` | Secret | Issuer ID of that API key |
| `MACOS_SIGNING_TEAM_ID` | Variable | Apple team ID that the signature must name |

When all inputs are absent, the job signs ad hoc. The log then says that the
asset is not for release. The job keeps that asset as the workflow artifact
`not-for-release-agent-macos`, and the release gets no macOS archive. When
only some inputs are present, the job fails.

[Agent guide](README.md) gives installation, commands, and code entry points.
