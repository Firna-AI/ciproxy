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

Release-plz raises the shared workspace version in a version pull request.
[Version pull requests](#version-pull-requests) gives the hand step that
each version pull request needs.

## When a shared library moves

The agent uses three shared libraries from other `futex-ai` repositories:
`cli-updater`, `cli-release-interface`, and `internal-error`. The root
`Cargo.toml` pins each of them to a Git commit in `[workspace.dependencies]`.
A shared library moves when its pin changes to another commit.

Release-plz does not see such a move. The move changes no file of the
agent, the cache sidecar, or the protocol crate, and the libraries keep
their version across commits. A version increase alone does not release
either:

- `release-plz.toml` sets `release_always = false`. So release-plz creates
  no tag and no release for a merge that does not come from a
  `release-plz-` branch. That prefix is the default of `pr_branch_prefix`,
  which `release-plz.toml` does not set.
- It also sets `changelog_update = false`. So release-plz opens no version
  pull request for a version change alone.
- Until the tag of the new version exists, release-plz opens no version
  pull request for later changes either.

So raise the agent version by hand when a shared library moves. Raise it in
a separate pull request from a `release-plz-` branch, so that the gated
`version` job releases it:

1. Move the pin in its own pull request.
2. After that pull request merges, create a branch from `main` whose name
   starts with `release-plz-`. Release-plz tags the last commit of this
   branch when GitHub merges it with a merge commit. So the branch must
   hold the moved pin.
3. On that branch, raise `version` under `[workspace.package]` in the root
   `Cargo.toml`.
4. Run `cargo build --workspace` to update `Cargo.lock`. Run
   `npm --prefix ts run api:generate` to update
   `ts/api/generated/openapi.json`.
5. Open a pull request from that branch with only these changes. Merge it.

After the merge, the `version` job runs only after the gate job passes on
the merge commit. Because the merge comes from a `release-plz-` branch,
release-plz then creates the tag `ciproxy-agent-vVERSION` and a draft
release. The tag starts the package jobs. The `publish` job then uploads the
archives and publishes the release.

Two cases change this route:

- When release-plz writes its own version pull request, it can take over
  the hand pull request from the `release-plz-` branch. It looks for
  commits from a person in the hand pull request. It skips the first
  commit, and it ignores logins that end in `[bot]`. So it takes over in
  one of two ways:
  - When no commit from a person follows the first commit, release-plz
    rewrites the hand pull request in place with its own version commit.
  - Otherwise, release-plz closes the hand pull request and opens a new
    version pull request.

  The version pull request that remains holds the moved pin, because the
  pin is already on `main`. It does not hold the regeneration of
  `ts/api/generated/openapi.json`. So do the steps of
  [Version pull requests](#version-pull-requests) on it.

- When the gate fails on the merge commit, no release follows. Release-plz
  releases the version when a rerun of that workflow run passes, or when a
  later merge from a `release-plz-` branch passes the gate.

[Release channels](../../deploy/release-server.md#release-channels) says
which environment serves a pre-release.

## Version pull requests

A version pull request raises the shared workspace version. Release-plz
opens one after a change to the agent, the cache sidecar, or the shared
protocol crate. A person opens one when a shared library moves.

The dashboard API crate `ciproxy-api` uses the shared workspace version
too. Its generated OpenAPI document, `ts/api/generated/openapi.json`, holds
that version in `info.version`. So each version raise changes that file.
Each version pull request must hold the changed file. Otherwise the
`api-types` phase of the required `Repository gate` check fails on it.

The steps of [When a shared library moves](#when-a-shared-library-moves)
regenerate the file before the pull request opens. Release-plz never
regenerates it. So do these steps on each version pull request from
release-plz:

1. Run `npm --prefix ts run api:generate` on the branch of the pull
   request.
2. Commit the changed generated files and push them.
3. Merge the pull request soon after its gate passes.

Release-plz can update an open version pull request after each later merge
to `main`. It counts the regeneration commit as a commit from a person. So
that update closes the pull request and opens a new one without the
regeneration. Then do the same steps on the new pull request.

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
