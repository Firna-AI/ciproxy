# Licence lines can contradict the directory licences

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`: the blocked wider fix E of
  finding 1105, with finding 705 of round 1 as its previous example.
- **Source:** Review.
- **Category:** Repository rule.
- **Location:** `xtask/src/check/phases.rs`, the phases of the gate,
  `xtask/README.md`, the phase table, `AGENTS.md`, the licence rule under
  "Repository rules", and `README.md`, "Licence".
- **Reason:** The user approved the gate phase of B on 19 November 2026,
  as work after the merge.

- **Terms:**
  - **Licensed directory:** a directory with its own Apache-2.0 `LICENSE`
    file. The "Licence" section of `README.md` lists them.
  - **Licence line:** a header comment that names a licence, such as
    `SPDX-License-Identifier: Apache-2.0` or a leading `Apache-2.0.`.
  - **Gate:** `cargo xtask check`, the repository gate with its phases.
    `xtask/README.md` lists the phases.
- **Background:**
  - The repository is public. Its open-source boundary is a set of
    directories with their own `LICENSE` files.
  - `AGENTS.md` says that each part that ships to the customer's side has
    its own Apache-2.0 `LICENSE` file. No directory with its own `LICENSE`
    file holds hosted code. The rest of the repository has no open licence.
  - The `LICENSE` file of a directory sets the licence of its files. A line
    in a file does not. So a reader does not have to open each file.
  - Files in licensed directories carry licence lines today. Examples are
    the SPDX lines of the Python and shell files under `image/`, and the
    `Apache-2.0.` lines of the smoke scripts in
    `crates/ciproxy-agent/scripts/`.
  - On 4 November 2026, commit `17d96b9` added the header line "Apache-2.0."
    to `scripts/cli-release.sh`. `scripts/` has no `LICENSE` file. At that
    time, `AGENTS.md` already gave the rest of the repository no open
    licence.
  - The conflict stayed until the final review of 17 November 2026 found
    it. Its round 2 fix moves the script into the licensed directory
    `crates/ciproxy-agent/scripts/`.
  - Product copy, docs, and `ts/package-lock.json` name Apache-2.0 in their
    content. So a check must read only header comments.
- **Problem:** No check compares the licence lines of files with the
  licence boundary. The written rule did not stop a file outside every
  licensed directory from naming Apache-2.0 for 13 days.
- **Impact if no action:** A later file can again name a licence that its
  directory does not give. Readers of the public repository then get two
  answers about the licence of that file.
- **Root question:** How does the repository stop a file from naming a
  licence that its directory does not give?
- **Options:** Only two approaches are real. A second written rule fails
  for the reason under **Left out**.
  - **A.** Keep the written rule only. `AGENTS.md` also names the
    open-source boundary under "Review focus", so each review checks it.
    Cost: the class stays possible between reviews, as the 13 days show.
    Effort: Small.
  - **B.** Add an `xtask` gate phase for licence lines.
    - The phase reads the header comment lines of each tracked file.
    - It fails when a file outside every licensed directory names a
      licence, for example with an `SPDX-License-Identifier:` line or a
      leading `Apache-2.0`.
    - It can also fail when a file in a licensed directory names a
      different licence.
    - It reads only header comments, because product copy, docs, and
      `ts/package-lock.json` name Apache-2.0 in their content.
    - The phase could instead forbid every licence line, so that only the
      `LICENSE` files give a licence. That rule is simpler, but it removes
      the licence lines that files under `image/` and the agent scripts
      carry today.
    - Add its tests and its row in `xtask/README.md`.
    - Cost: it is a new gate, so the user must approve it. It needs header
      rules for several comment styles.
    - Effort: Medium.
- **Left out:** Another written rule, in `AGENTS.md` or a shared skill: it
  fails the goal. The rule of `AGENTS.md` was in force on 4 November 2026
  and did not prevent the class.
- **Wider fix:** B. It catches a licence line outside the boundary in any
  file and in any header style.
- **Previous examples:** Round 1 of the same review found that `AGENTS.md`
  named only two of the six licensed directories, and called the rest of
  the repository closed. Commit `2a61e1c` of 17 November 2026 rewrote the
  rule by criterion. The second review pass of that round also first found
  the header of `scripts/cli-release.sh`.
- **Recommendation:** B. The written rule did not prevent the class, and B
  catches it in every file at each gate run.
- **Action:** fix after merge, apply B, because the user approved this new
  gate phase on 19 November 2026, as work after the merge.
