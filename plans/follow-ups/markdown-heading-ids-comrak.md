# Gate heading IDs differ from GitHub

- **Severity:** Medium. **Effort:** Large. **Confidence:** High. **Harder
  later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-04, from pull request #2, branch
  `conductor/ciproxy-mvp-foundation-m4-bootstrap`, the wider fix of review
  finding 1.
- **Source:** Implementation.
- **Category:** Repository rule.
- **Location:** `xtask/src/markdown/parser.rs`,
  `xtask/src/markdown/links.rs`,
  `xtask/src/markdown/_tests_/links_tests.rs`, `xtask/Cargo.toml`, and
  `Cargo.lock`.
- **Reason:** The selected replacement adds refusals, including two valid
  GitHub links. The user requires GitHub compatibility and no new refusal.
  These requirements conflict where the current gate accepts invalid
  links. Open pull request #39 owns `Cargo.lock`. The user excludes files
  of open pull requests from this branch. Options that change dependencies
  must wait until that constraint clears or the user changes it.
  Options A and E need no dependency change. No merge of #39 is requested.

- **Terms:**
  - **Heading ID:** the anchor that GitHub gives a Markdown heading, which a
    link fragment such as `#intro-2` targets.
  - **comrak:** a maintained Rust port of GitHub's cmark-gfm Markdown
    parser.
  - **False acceptance:** a link that passes the gate but has no target
    anchor in GitHub's rendered document.
  - **HTML parser:** a maintained library that reads rendered HTML into
    elements and text.
  - **Sanitizer:** GitHub's filter that removes HTML it does not permit.
- **Background:** The gate validates local Markdown paths and heading
  fragments through `links::violations`. It currently uses pulldown-cmark
  with all extensions except wikilinks, followed by a hand-written slug.
  On 4 November 2026, the user selected option B below. The required
  behaviour is GitHub compatibility without any new refusal.
- **Problem:** The current parser changes heading attributes and smart
  punctuation that GitHub treats as literal heading text. Its false
  acceptances make exact GitHub compatibility conflict with preserving
  every current acceptance. Comrak alone also rejects valid GitHub links.
  A direct replacement therefore fails both requirements.
- **Impact if no action:** Valid GitHub links can fail the current gate.
  Invalid heading links can also pass. Shipping the direct replacement
  would add incorrect failures for valid GitHub links.
- **Root question:** How does the gate give each local link the same
  verdict that GitHub's rendered page gives?
- **Options:**
  - **A.** Fix each slug and parser case by hand in the current code.
    Effort: Medium.
  - **B.** Take the heading IDs from comrak, with the GFM extensions and
    `---` front matter, in place of pulldown-cmark and the hand-written
    slug. Take the expected IDs of the tests from GitHub's renderer.
    Comrak 0.59 adds 14 packages with default features disabled.
    The direct replacement fails the existing valid-links test.
    Effort: Medium.
  - **C.** Replace the Rust link check with the Node checker
    `remark-validate-links`, which uses `github-slugger`. It adds a Node
    step to the gate and reports missing images, a new rejection. Effort:
    Medium.
  - **D.** Complete the maintained heading model before replacing the
    parser. Fix the known compatibility gaps through upstream changes or
    another maintained renderer component. Permit rejection of proven
    GitHub-invalid local Markdown links that the current checker accepts.
    Keep valid GitHub links accepted and retain the explicit exceptions
    below. The replacement waits for those checks and dependency-file
    ownership. Effort: Large.
  - **E.** Keep pulldown-cmark, with only GFM extensions, math, alerts, and
    `---` metadata. Include math text in headings and omit image alt text.
    This needs no dependency or lockfile change. Source reading suggests
    it corrects several cases and preserves valid line-break IDs.
    Footnote numbers and HTML headings and anchors remain gaps.
    It still needs approval for new refusals. Effort: Small.
  - **F.** Keep current anchors as aliases beside a GitHub model from E or
    G. Fail a link only when both models fail it. Warn when only the
    GitHub model fails it. This preserves acceptance but relaxes exact
    GitHub matching. It keeps two models and needs approval for aliases.
    Effort: Medium.
  - **G.** Render with comrak, then use a maintained HTML parser to extract
    heading text and explicit `id` and `name` anchors. Generate heading IDs
    with comrak's `Anchorizer`. Match which anchors GitHub's sanitizer
    keeps. Source reading suggests this can address the known gaps.
    No completed implementation or test proves that result.
    Sanitizer compatibility remains unverified. This needs `Cargo.lock`
    and approval for new refusals. Effort: Large.
- **Left out:** A live GitHub request in the gate adds a network dependency
  and can fail without a code defect. Compatibility aliases return as F,
  because the conflicting requirements make that trade-off a user choice.
  Checking only verdicts for current files leaves the demonstrated defects.
- **Wider fix:** G. Extracting text from rendered headings can cover new
  heading constructs without a hand-written text rule for each construct.
  This remains a proposed design until renderer and sanitizer checks pass.
- **Previous examples:** Pull request #2 fixed heading collisions and the
  symbol slug rule. The older investigation checked 30 cases and reported
  6 matches for the gate and 23 for comrak. The newer check below uses
  different specimens and supersedes that result for implementation work.
- **Related:** Pull request #39 reserves `Cargo.lock`. It blocks options
  that change that file, independently of the acceptance decision.
  It does not block A or E.
- **Recommendation:** G. It keeps the approved comrak direction and can
  address the known gaps after rendering. The initial recommendation was
  D. Implementation and sanitizer checks remain required.
- **Action:** ask the user, because exact GitHub compatibility adds
  refusals that the user has explicitly prohibited.

## Preserved choices

- On 4 November 2026, the user chose B as the parser replacement approach.
  Its earlier recommendation relied on unchanged verdicts for current
  files. That evidence does not prove that it preserves all acceptances.
- On 4 November 2026, the user kept accepting heading fragments on
  `?plain=1` links. Keep that exception, although the plain view has no
  heading anchors.
- On 4 November 2026, the user kept markdownlint's MD051 rule unchanged.
  This work changes neither that rule nor its invocation.
  MD051 also checks same-file fragments with its own heading rules.
  Source reading shows differences for attributes, metadata, and HTML
  headings. No option alone makes same-file links exact while MD051 stays.
- The local link check does not inspect paths in raw HTML `href` values.
  Keep that scope. Checking those paths needs separate approval.
- Add no gate phase, image check, stricter path check, or compatibility
  alias without the user's approval.

## Checked on 7 November 2026

The check ran an unchanged copy of the production local link validator.
It changed only the parser in the proposed copy.
GitHub's anonymous Markdown renderer supplied the literal expected IDs.
The two regression tests called `links::violations`, not a parser helper.

- Both new tests failed on assertions with the current parser.
  The test of valid GitHub IDs reported three missing fragments.
  The test of invalid old IDs reported no violations instead of three.
- Both tests passed with the proposed comrak parser.
  This proves the correction also adds three refusals.
- All eight targeted Markdown tests ran against the proposed parser.
  Seven passed, and the existing valid-links test failed.
  It rejected `#softbreak` and `#hardbreak`, which GitHub renders as valid.
  Comrak instead produces `#soft-break` and `#hard-break`.
- A fixed repository snapshot had 113 Markdown documents, 625 heading
  anchors, and 1,106 link occurrences. Both parsers produced identical
  output and no violations for that snapshot. This does not establish
  preservation for other documents.
- A separate comparison used 30 renderer specimens and 47 candidate links.
  The direct replacement added seven fragment refusals and eight fragment
  acceptances. Five new refusals remove current false acceptances.
  Two new refusals reject valid GitHub links. Two new acceptances are also
  invalid: `#soft-break` and `#hard-break`.
- It also exposed two missing-file failures that the current parser hides.
  One link occurs inside a heading attribute.
  The other link occurs inside `+++` metadata, which GitHub treats as text.
  The selected front-matter extension covers only `---`.
- Comrak still differs for image alt text, footnote numbers, HTML headings,
  HTML `id` and `name` anchors, and soft and hard line breaks.
  Those gaps prevent the direct prototype from being a complete fix.
- The symbol specimen verified removal of `👋`, `→`, and `©` only.
  It did not verify other Unicode characters.
- GitHub's raw Markdown API does not apply blob-view YAML front matter.
  A separate public blob render confirmed that GitHub treats `---`
  metadata separately. Do not use raw-mode front-matter output as a
  blob-view expectation.

## Source-reading concerns

The second-pass options check read the sources and saved evidence.
It ran no new implementation, Cargo command, or GitHub renderer request.
Its claims for E and G therefore remain proposals, not checked results.

- The current slug may keep zero-width joiners and drop `²`.
- The current parser may accept `---` metadata after the first block.
  That could hide later headings and links.
- Neither concern has a GitHub comparison or a regression proof.
  Check them during implementation before reporting new regressions.

## Concrete verdict changes

| Heading or document | GitHub target | Current gate | Direct comrak |
| --- | --- | --- | --- |
| `# Foo {#custom}` | `#foo-custom` | Accepts invalid `#custom` | Refuses `#custom` |
| `# Foo -- Bar` | `#foo----bar` | Accepts invalid `#foo--bar` | Refuses `#foo--bar` |
| `# Cost $a+b$ Done` | `#cost-ab-done` | Accepts invalid `#cost--done` | Refuses `#cost--done` |
| `Soft`, newline, `Break`, newline, `---` | `#softbreak` | Accepts | Refuses |
| `Hard\`, newline, `Break`, newline, `---` | `#hardbreak` | Accepts | Refuses |
| `# Foo {key="[Missing](gone.md)"}` | Missing file | Accepts document | Refuses document |
| `+++`, newline, `[Missing](gone.md)`, newline, `+++` | Missing file | Accepts document | Refuses document |

The current parser accepts the invalid IDs above.
Accepting GitHub's correct IDs as additional aliases would leave those
false acceptances in place.
Rejecting them would be a new refusal.
The user must resolve that conflict before this replacement can proceed.

## Sources

- [GitHub section links and custom anchors][github-links]
  define the rendered targets.
- [GitHub Markdown API](https://docs.github.com/en/rest/markdown/markdown)
  documents anonymous rendering of public content.
- [Comrak 0.59 options][comrak-options]
  define the enabled GFM and front-matter extensions.
- [Comrak text collection][comrak-text]
  includes image text and replaces line breaks with spaces.

[github-links]: https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#section-links
[comrak-options]: https://docs.rs/comrak/0.59.0/comrak/options/struct.Extension.html
[comrak-text]: https://docs.rs/comrak/0.59.0/src/comrak/nodes.rs.html

- **User answer, 2026-11-08:** item 97 = G, as a follow-up item that
  blocks nothing. Until then, fix a link test that breaks by hand.
  **Action:** fix after merge, apply G.
- **Interim, 2026-11-08:** pull request #49 applies option I, which the user
  chose on 7 November 2026: a fragment that matches no heading only warns,
  and a link to a missing file still fails. G stays this follow-up item.
