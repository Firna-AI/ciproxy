# Preview install commands select the production endpoint

- **Severity:** High. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-08, from the release-server pin and channel review.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-api/src/handlers/machines.rs:206`,
  `crates/ciproxy-agent/src/application.rs:171`, and
  `crates/ciproxy-agent/src/cli.rs:15`.
- **Reason:** This existing defect needs API and agent changes that other
  workspaces own. The release-server change does not change those files.
- **Terms:**
  - **Agent endpoint:** the WSS route for the agent's control-plane connection.
  - **Install command:** the shell command that the dashboard gets from the
    API to install and register an agent.
- **Background:** The API returns an install command with the environment's
  download URL, followed by `install --token`.
  The agent uses `CIPROXY_AGENT_URL` for its endpoint.
  Without this setting, it uses `wss://app.ciproxy.dev/agent/`.
  The CLI install action accepts a token but no endpoint option.
  The agent README documents the environment override.
- **Problem:** The generated preview command supplies no agent endpoint.
  A new preview machine thus attempts registration against production.
  Selecting the preview download host does not change the agent endpoint.
- **Impact if no action:** The copied preview command cannot register the
  machine with its preview control plane unless the operator supplies the
  documented override.
- **Root question:** How does a copied install command register with the
  control plane that issued it?
- **Options:**
  - **A.** Include the issuing environment's endpoint through the existing
    agent setting. Preserve it for the installed service. Derive the value
    from trusted runtime configuration. Effort: Medium.
- **Left out:** Manual edits to each copied command leave the dashboard
  command wrong. Changing the shared default to preview breaks production.
  A separate preview binary adds release work that the existing setting
  avoids. These alternatives do not improve the copied command over A.
- **Wider fix:** A. The issuing environment supplies the endpoint for both
  registration and the installed service.
- **Previous examples:** `crates/ciproxy-agent/host-preparation.md`
  documents a root-shell staging override.
  `crates/ciproxy-agent/scripts/smoke-host.sh` persists that override before
  direct binary installation. Those operator paths do not repair the
  dashboard command.
- **Related:** `plans/follow-ups/release-server-prerelease-channel.md`
  records the release-server rollout. That rollout can use the documented
  root-shell override while this separate caller fix waits.
- **Recommendation:** A. It uses the existing agent setting and keeps one
  binary for both environments.
- **Action:** fix after merge, apply A, because the defect is outside the
  release-server scope and the user assigned API and agent work elsewhere.
