# Markdown checks skip the crate guides

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from pull request #142
  (`conductor/ciproxy-runner-group-per-environment`), the check of review
  finding 8.
- **Location:** `xtask/src/markdown/files.rs:5`, `xtask/README.md`,
  section "What This Crate Does".
- **Reason:** A wider Markdown check is a new gate, and only the user adds
  a gate.

- **Terms:**
  - **Crate guide:** a Markdown file beside a crate's `README.md` that is
    not itself a `README.md`, such as `crates/ciproxy-github/scale-sets.md`.
  - **Gate:** `cargo xtask check`, the repository check that CI runs on
    each push and pull request.
- **Background:** The function `maintained` in
  `xtask/src/markdown/files.rs` selects the Markdown files of the gate's
  `markdown` phase. It selects `AGENTS.md`, every `README.md`, Markdown
  under `docs/` and `plans/`, and the Markdown of `ts/site/`. The phase
  checks the local links of those files, then runs the pinned
  `markdownlint-cli2`. `xtask/README.md` states this list. On
  23 November 2026 the repository had 31 crate guides that the phase
  skips. Their local links all resolved then.
- **Problem:** No gate checks the local links or the lint rules of a crate
  guide. A guide can link a moved heading or file, or break the line
  rules, and the gate still passes.
- **Impact if no action:** Broken links and lint problems in crate guides
  stay unseen until a reader finds them. Each new guide adds to the work
  of a later fix.
- **Root question:** Should the gate check the Markdown of crate guides
  in the same way as each `README.md`?
- **Options:**
  - **A.** Select every Markdown file under `crates/` and `xtask/` in
    `maintained`, and fix what the lint then reports. Update
    `xtask/README.md`. Effort: Medium.
  - **B.** Select crate guides for the link check only, not for the lint.
    Effort: Small.
  - **C.** Keep the gate, and check crate guides by hand when a change
    edits one. Effort: Small.
- **Left out:** None.
- **Wider fix:** A. One rule then covers every maintained Markdown file.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/docs-crate-guides-in-spec.md` asks for a
  `documentation` rule that sends writers to the content of crate guides.
  This item covers only their links and their lint.
- **Recommendation:** A. It checks the guides like the docs that they sit
  beside.
- **Action:** ask the user, because options A and B add a new gate.
