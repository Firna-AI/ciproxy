# Labels page leaves out Boat-only labels

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch `calummoore/diagram-request-3jpv`,
  the check of review finding 1.
- **Source:** Review.
- **Category:** Docs or spec.
- **Location:** `ts/site/src/content/docs/docs/reference/labels.mdx:19-21`
  and its mockup `docs/mockups/src/screens/docs/labels-reference.tsx:105-108`.
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
  - The page How ciproxy works now names the jobs of labels that allow only
    Boat, since the branch that added its diagram.
- **Problem:** The labels page gives one rule for every label. A label with
  "Your machines" off sends its jobs to Boat at once, also while a machine
  has room.
- **Impact if no action:** A customer can take an expected paid Boat start
  for a routing error.
- **Root question:** How does the labels page say where the jobs of each
  kind of label run?
- **Options:**
  - **A.** Add the Boat-only case to the sentence on the page and in its
    mockup, in the words of How ciproxy works: "and the jobs of labels that
    allow only Boat". Effort: Small.
  - **B.** Link the sentence to "Overflow to Boat" on How ciproxy works, and
    keep the short rule. The page then still states a narrow rule. Effort:
    Small.
- **Left out:** Change nothing, because the page contradicts the protocol.
- **Wider fix:** None. The `documentation` skill already requires docs to
  agree with the product.
- **Previous examples:** Review finding 1 of the branch
  `calummoore/diagram-request-3jpv`, on How ciproxy works.
- **Recommendation:** A. It states the full rule in one clause.
- **Action:** fix after merge, apply A, because the lines are outside the
  diff of the branch that found them.
