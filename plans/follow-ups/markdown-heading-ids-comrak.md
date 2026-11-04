# Gate heading IDs differ from GitHub

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High. **Harder
  later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-04, from pull request #2, branch
  `conductor/ciproxy-mvp-foundation-m4-bootstrap`, the wider fix of review
  finding 1.
- **Location:** `xtask/src/markdown/parser.rs` and the Markdown link check
  of the gate under `xtask/src/markdown/`.
- **Reason:** Pull request #2 applies the direct fix for heading-ID
  collisions. This wider fix replaces the parser and adds a dependency, so
  it goes into its own branch.

- **Terms:**
  - **Heading ID:** the anchor that GitHub gives a Markdown heading, which a
    link fragment such as `#intro-2` targets.
  - **comrak:** a maintained Rust port of GitHub's cmark-gfm Markdown
    parser.
- **Background:** The gate checks that every Markdown link fragment points
  to a heading. It builds heading IDs with pulldown-cmark and a hand-written
  slug function. A check ran 30 cases through GitHub's Markdown API: the
  gate's code matched GitHub on 6, and comrak 0.59 matched on 23,
  including all collision cases. On the current Markdown files, comrak
  gives the same IDs as the gate, so no current link changes its verdict.
- **Problem:** The gate's IDs differ from GitHub's in several cases:
  - The slug keeps emoji and symbols that GitHub drops, such as `👋`, `→`,
    and `©`.
  - Smart punctuation and heading attributes change IDs: `# Foo -- Bar`
    gives `foo--bar`, not `foo----bar`.
  - Heading text drops inline math and footnote references, and keeps
    image alt text.
  - Raw HTML headings and HTML `id` anchors get no IDs, so links to them
    fail.
- **Impact if no action:** Valid links fail the gate, and contributors must
  rename valid headings to pass.
- **Root question:** How does the gate get the same heading IDs as GitHub?
- **Options:**
  - **A.** Fix each slug and parser case by hand in the current code.
    Effort: Medium.
  - **B.** Take the heading IDs from comrak, with the GFM extensions and
    `---` front matter, in place of pulldown-cmark and the hand-written
    slug. Take the expected IDs of the tests from GitHub's renderer. About
    15 crates replace 2. Effort: Medium.
  - **C.** Replace the Rust link check with the Node checker
    `remark-validate-links`, which uses `github-slugger`. It adds a Node
    step to the gate and reports missing images, a new rejection. Effort:
    Medium.
- **Left out:** A live call to GitHub's Markdown API in the gate, because
  the gate would need the network and a token and would be flaky.
- **Wider fix:** B. A maintained GitHub-compatible parser owns the heading
  model, so a change in a slug or parser detail needs no hand edit.
- **Previous examples:** The collision case, fixed directly in pull
  request #2. The symbol case, review finding 6 of the same pull request,
  fixed directly with GitHub's slug rule. Also open in the same checker: a
  heading fragment on a `?plain=1` link passes, although GitHub's plain
  view has no heading anchors. Rejecting it is a new rejection, which needs
  the user.
- **Related:** Links in the same file go through markdownlint's MD051
  rule, which this fix does not change. On 4 November 2026 the user kept
  that rule as it is.
- **Recommendation:** B. One maintained dependency fixes most cases, and no
  current link changes its verdict.
- **Action:** fix after merge, apply B, because it replaces the parser of
  the gate and adds a dependency, which a separate branch reviews better.
