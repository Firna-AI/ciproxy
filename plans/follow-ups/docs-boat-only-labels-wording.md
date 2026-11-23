# Docs pages narrow the Boat routing rule

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch `calummoore/diagram-request-3jpv`,
  the check of review finding 1.
- **Source:** Review.
- **Category:** Docs or spec.
- **Location:** `ts/site/src/content/docs/docs/reference/labels.mdx:19-21`
  and `:125`, `ts/site/src/content/docs/docs/guides/connect-boat.md:44` and
  `:76`, `ts/site/src/content/docs/docs/guides/set-a-spend-limit.md:44`, and
  the mockup `docs/mockups/src/screens/docs/labels-reference.tsx:105-108`,
  `:195`, and `:201`.
- **Reason:** The branch that found it changes only the page How ciproxy
  works, and these lines were already on `main`.
- **Terms:**
  - **Boat:** the provider that runs overflow jobs in sandboxes of the
    customer's own Boat account.
- **Background:**
  - Each Linux label allows own machines, Boat, or both, with the switches
    "Your machines" and "Boat" of its dialog, as
    `docs/protocol/labels.md` says under "Own machines, Boat, or both".
  - When no own machine can run a job, for example because the label has
    "Your machines" off, ciproxy starts a Boat runner at once, as
    `docs/protocol/job-routing.md` says. `crates/ciproxy-scaler/src/overflow.rs`
    does this.
  - The page Labels and VM sizes says that each job starts "on your
    machines first, and on Boat when they are busy". Its mockup says "when
    they are full".
  - The Overview page says that ciproxy "can" start a Boat runner when the
    machines have no room. That sentence is not wrong, so it stays.
  - `labels.mdx:125`, `connect-boat.md:76`, `set-a-spend-limit.md:44`, and
    `labels-reference.tsx:201` say that jobs run on your machines while Boat
    cannot take them. A job whose label allows only Boat never runs on your
    machines. It stays queued in GitHub, as `docs/protocol/job-routing.md`
    says.
  - `connect-boat.md:44` and `labels-reference.tsx:195` start a job on Boat
    at once or after the wait, with no exception for a macOS job. A macOS
    job never runs on Boat.
  - "Overflow to Boat" on How ciproxy works states every case, since the
    branch that added its diagram. Its summary in "The parts" says: "Your
    Boat account runs a job when your machines cannot take it in time and
    Boat can take it."
- **Problem:** These pages give one Boat rule for every label. A label with
  "Your machines" off sends its jobs to Boat at once, also while a machine
  has room, and waits in GitHub when Boat cannot take them. A macOS job
  never goes to Boat.
- **Impact if no action:** A customer can take an expected paid Boat start
  for a routing error.
- **Root question:** How do these pages say when a job runs on Boat and
  when it waits?
- **Options:**
  - **A.** Give each sentence the summary of How ciproxy works, "when your
    machines cannot take it in time and Boat can take it", and link it to
    "Overflow to Boat". Make the same edits in the mockup. Effort: Small.
  - **B.** State every case in each sentence. The rule then lives in six
    places. Effort: Small.
- **Left out:** Change nothing, because the page contradicts the protocol.
- **Wider fix:** None. The `documentation` skill already requires docs to
  agree with the product.
- **Previous examples:** Review finding 1 of the branch
  `calummoore/diagram-request-3jpv`, on How ciproxy works.
- **Recommendation:** A. One section holds the rule, and the other pages
  link to it.
- **Action:** fix after merge, apply A, because the lines are outside the
  diff of the branch that found them.
