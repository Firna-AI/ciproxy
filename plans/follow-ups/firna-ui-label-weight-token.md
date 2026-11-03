# Firna controls force bold labels

- **Severity:** Low. **Effort:** Medium. **Confidence:** High. **Harder later:**
  No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-03, from the branch `calummoore/ci-proxy-load-balancer`,
  Milestone 1 design directions.
- **Location:** The `Button` and `SegmentedControl` components of the npm
  package `@firna/ui` 4.0.0. The ciproxy mockups use them under
  `docs/mockups/src`.
- **Reason:** The fix belongs to the `@firna/ui` repository, not to ciproxy.

- **Terms:**
  - **`@firna/ui`:** the shared React Native and web UI library of the Firna
    products, published on npm.
  - **Design direction:** one complete visual style of the ciproxy dashboard
    mockups: palette, fonts, corner radius, density, and chrome.
- **Background:** The ciproxy dashboard mockups reuse the Firna buttons and the
  segmented control, for example the period control on the Savings screen. Firna
  draws the labels of these controls with inline styles. The professional
  directions for ciproxy, modelled on Vercel and Linear, use a label weight of
  about 500.
- **Problem:** Firna fixes the label weight of these controls at 700 and has no
  theme token for it. So ciproxy buttons look slightly heavier than the rest of
  the chosen direction.
- **Impact if no action:** The buttons stay a little heavy. The rest of the look
  does not change.
- **Root question:** How do the shared controls take the label weight of the
  theme of each product?
- **Options:**
  - **A.** Keep weight 700. No work, and the visual cost is small. Effort:
    Small.
  - **B.** Add a label-weight token to the Firna theme, and read it in `Button`
    and `SegmentedControl`. This changes the shared library in its own
    repository. Effort: Medium.
  - **C.** Draw buttons in the ciproxy mockups and product only. This moves away
    from the shared library. Effort: Medium.
- **Left out:** CSS overrides with `!important`, because they fight the inline
  styles and break on each Firna release.
- **Wider fix:** B, because one token fixes the weight for every Firna product.
- **Previous examples:** None found.
- **Recommendation:** B. One theme token fixes the weight for the mockups and
  for the product.
- **Action:** fix after merge, apply B, because the change belongs to the
  `@firna/ui` repository, outside this branch.
