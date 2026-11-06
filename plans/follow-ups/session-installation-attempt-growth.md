# Session reads create permanent installation records

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from PR #32, owner review round 1, finding 3.
- **Source:** Review.
- **Category:** Performance.
- **Location:** `crates/ciproxy-api/src/handlers/sessions.rs:147`,
  `crates/ciproxy-api/src/handlers/links.rs:21` and `:49`, the functions
  `install_link` and `continuation_link`,
  `crates/ciproxy-store-pg/src/installation_links/attempts.rs:25` and `:45`,
  the functions `save` and `pending`, and
  `crates/ciproxy-store-pg/src/installation_links/retention.rs:16`.
- **Reason:** This database growth can be corrected separately from the
  linking fixes.

- **Terms:**
  - **Attempt:** a row of the `installation_attempts` table that binds the
    state of one GitHub install link to the person who received it.
  - **Direct install:** a normal install, which makes a new workspace or
    returns a retained organization to the workspace that it left.
  - **Add:** an install that an Admin starts with "Add an organization",
    which joins the workspace that the person chooses after the return.
  - **Install context:** the part of a sign-in that names an exact attempt,
    or an ambiguous Add after a setup return without state.
  - **Late approval:** an owner approves the install request of a member on
    GitHub later, and that event carries no state from ciproxy.
- **Background:**
  - Every session response supplies an installation link, in the field
    `install_url` of `SessionView` in `crates/ciproxy-api/src/wire/access.rs`.
    Its doc says "Install entry point when no workspace is available".
  - `session` in `sessions.rs` builds that link through `continuation_link`.
    Each path of `continuation_link` ends in `install_link`.
  - `install_link` makes a new state secret and calls
    `begin_installation_attempt`. The store runs a plain `INSERT` with no
    installation ID, in `save` of `attempts.rs`.
  - So each `GET /session` inserts one attempt. With no install context,
    the attempt is Direct. In an Add context or an ambiguous Add context, it
    is Add.
  - The migration
    `crates/ciproxy-schema/migrations/202611062300_installation_links/up.sql`
    creates `installation_attempts` with no creation time and no foreign
    key.
  - The only delete of attempts is in `purge` of `retention.rs`. It removes
    only attempts bound to an installation of a deleted, linked
    organization. Sign-out removes nothing.
  - Before the change that added this table, the session returned the
    static `github_install_url` of the API configuration. The old table
    `dashboard_install_intents` kept one row for each person and
    organization, so it stayed bounded.
  - Add evidence must be durable before GitHub opens, and it has no age
    limit. The install event can arrive before the browser returns, and a
    late approval carries no state.
  - `docs/protocol/workspaces.md`, in "Choose the workspace" and "A request
    that an owner approves later", states these rules. The store interface
    in `crates/ciproxy-store-interface/src/installation_links.rs` says "Save
    entry evidence before exposing the GitHub install URL".
  - `pending` in `attempts.rs` backs `has_pending_add`. It is true while the
    person has an Add attempt with no installation.
  - Signed association reads the same `add_evidence` query for the original
    requester, in
    `crates/ciproxy-store-pg/src/installation_links/association.rs`.
    An unresolved or exact Add attempt keeps that install in Add recovery.
    The API authority consumes the canonical claim and resolves its Target
    approval and billing holds.
  - A continuation keeps the purpose of its first attempt. A retry after a
    personal-account return stays Add, as `setup_personal_tests.rs`
    asserts. After its installation links, an Add continuation offers a
    Direct link.
  - `link_flow_tests.rs`, `link_recovery_tests.rs`, and
    `setup_personal_tests.rs`, under `crates/ciproxy-api/tests/http/`, take
    the GitHub state from the `install_url` of a session.
  - `protect` in `crates/ciproxy-api/src/router.rs` requires the Origin and
    the CSRF header of the session for each request that is not GET, HEAD,
    or OPTIONS.
  - The change that added this item also gave Direct and Add installs
    distinct recovery states, with their own proof rules.
  - The existing session and retention tests do not check whether repeated
    session reads leave unused attempts.
- **Problem:** Every `GET /session` creates another attempt, even when the
  person never starts an installation. These records have no installation
  ID, and cleanup deletes only attempts associated with deleted, linked
  organizations. In an Add context, the unused Add attempts also keep
  `has_pending_add` true after that Add completes.
- **Impact if no action:** Normal dashboard reads permanently increase the
  database size, and neither sign-out nor the existing retention cleanup can
  remove these unused records. A later install event of that person that
  carries no state waits for a workspace choice, instead of a Direct
  install.
- **Root question:** How should session reads avoid creating permanent
  installation work?
- **Options:**
  - **A.** Reuse one attempt for each browser continuation. Preserve its
    exact installation purpose. Each sign-in in an Add context still creates
    an Add attempt that can stay unused. Effort: Medium.
  - **B.** Return a stable navigation link from the session. Create durable
    evidence only when the person starts installation. It changes the
    meaning of `install_url` and adds a start action. Effort: Medium.
  - **C.** Use signed state for Direct links. Keep durable attempts for
    explicit Add operations. Each read in an Add context still creates an
    Add attempt. Effort: Medium.
  - **D.** Keep at most one open attempt for each person and purpose, with a
    partial unique index. The session returns the link of that attempt.
    Derive its state from the attempt ID with a server key, or seal the
    state, so ciproxy can build the link again. It needs a migration. Two
    tabs then share one state, so the second return fails the
    bound-installation check. Effort: Medium.
  - **E.** Do B. Also delete each Direct attempt that GitHub does not return
    within a fixed time. Keep Add attempts until their install arrives. It
    needs a migration for the creation time. An expired Direct return then
    fails, which is a new rejection, so it needs the user. Effort: Medium.
- **Left out:**
  - A simple age limit cannot distinguish an unused Add link from an
    installation awaiting late owner approval. E limits only Direct
    attempts.
  - Signed state for Add evidence breaks a requirement that stays. The event
    can arrive before the return, and the event order chooses nothing.
  - Change nothing fails the goal, because each read adds a row that nothing
    removes.
  - A read-only store for GET handlers is not a real choice, because
    `GET /session` refreshes Admin roles on purpose.
- **Wider fix:** B. It ties durable records to explicit installation
  actions, so no read path can create one.
- **Previous examples:** None found.
- **Recommendation:** B. Creating records only when installation starts best
  protects the codebase.
- **Action:** fix after merge, apply B, because this database growth can be
  corrected separately from the linking fixes.
  - `GET /session` returns a stable navigation link and writes no attempt.
  - An explicit start action creates the attempt and returns the GitHub
    install URL with its state.
  - The start action reads the install context of the session. An Add
    continuation stays Add until its installation links, then continues as
    Direct, as `continuation_link` does now.
  - Make the start action a POST, so the Origin and CSRF guard of `protect`
    applies. A cross-site GET must not create Add evidence.
  - Keep Add evidence durable before GitHub opens, and keep the rules of a
    late approval. Create the evidence in the Direct and Add recovery
    states of the change that added this item.
  - B adds no time limit. A time limit for Direct attempts is option E,
    which needs the user.
  - Add a regression with the real PostgreSQL store. Read `GET /session`
    several times and expect no new attempt. Then start an install through
    the start action, and expect one attempt with the purpose of the
    session.
  - Move the three tests that read `install_url` to the start action.
  - Regenerate the API artifacts with `npm --prefix ts run api:generate`
    for the new start operation and the session contract. A new HTTP path
    also changes the OpenAPI document.
