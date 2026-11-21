# Licence headers can escape directory boundaries

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-17, from final-review finding 1105, wider option E.
- **Source:** Review.
- **Category:** Repository rule.
- **Location:** `AGENTS.md`, `README.md#licence`, and `xtask/README.md`.
- **Reason:** A new gate needs the owner's approval.

- **Terms:**
  - **Directory licence:** a `LICENSE` file that defines the open-source
    boundary for its directory.
  - **Header:** the leading comment lines of a source file.
- **Background:** Customer-side code has an Apache-2.0 directory licence.
  The rest of the repository has no open licence. The release helper
  declared Apache-2.0 while it lived outside a licensed directory.
  Moving it under `crates/ciproxy-agent/scripts/` fixes that instance.
- **Problem:** The directory rule did not prevent a conflicting header.
  No repository check detects the same conflict in another file.
- **Impact if no action:** Review must find later licence conflicts by
  hand.
- **Root question:** How should the repository detect conflicting licence
  headers?
- **Options:**
  - **F.** Keep the directory rule and manual review. Add no gate.
    Effort: Small.
  - **E.** Add an xtask gate phase that checks tracked header comments
    against their containing directory licences. Keep the source option's
    letter. Effort: Medium.
- **Left out:** Searching complete files gives false results from product
  copy, docs, and dependency lockfiles. Licensing every directory would
  change the hosted-code boundary.
- **Wider fix:** E. It checks this class of conflicting headers.
- **Previous examples:** Installer and updater licence declarations once
  lacked directory licence files. The release helper repeated this class.
- **Related:** The direct release-helper move is complete. This gate is
  independent of that move.
- **Recommendation:** E. A header-aware check detects later conflicts
  without treating ordinary licence references as declarations.
- **Action:** ask the user, because E adds a new gate. A later change must
  define supported header styles, add focused tests, and document the phase.
