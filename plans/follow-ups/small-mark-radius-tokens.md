# Small data marks lack radius tokens

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-04, from
  `conductor/ciproxy-mvp-milestone-23-site`, review finding 2.
- **Location:** `ts/tokens/index.ts`, `ts/tokens/css.ts`,
  `ts/site/src/components/marketing/hero-panel.css`,
  `ts/site/src/components/marketing/cost.css`, and the five mockup rules
  listed below.
- **Reason:** The site task excludes the token package and mockups, while
  future dashboard work will copy the same shapes.

- **Terms:**
  - **Small mark:** A slot segment or a bar end in a data display.
  - **Token:** A design value owned by `@ciproxy/tokens` and shared by the
    site, dashboard, and mockups.
  - **Firna:** The shared component library used by the dashboard and
    mockups. Its theme consumes the existing radius map.
- **Background:** The approved mockups use 1 px, 2 px, and 3 px radii on
  small marks. The shared radius scale starts at 4 px. The site preserves
  the approved shapes with `calc(var(--r-sm) / 2)` and
  `calc(var(--r-md) / 2)` instead of literal radii.
- **Problem:** The token package has no named values for these shapes.
  Five older mockup rules still use literals. Future dashboard consumers
  must either copy those values or repeat the site's arithmetic.
- **Impact if no action:** More consumers can copy values outside the
  token source. A later design change then needs changes in each copy.
- **Root question:** How do small marks obtain their approved corner
  radii from one shared source?
- **Options:**
  - **A.** Derive the two site values from existing tokens. This preserves
    today's shapes but leaves other consumers to repeat the arithmetic.
    Effort: Small.
  - **B.** Add named 2 px and 3 px tokens and use them in the site. This
    leaves the existing 1 px marks and mockup copies. Effort: Small.
  - **C.** Add a separate small-mark radius group with 1 px, 2 px, and
    3 px values. Expose its CSS properties and use it in the site and five
    mockup rules. Keep the existing Firna radius map unchanged. Let the
    dashboard use the same group. Effort: Small.
- **Left out:** Keeping literals breaks the token-source rule. Using
  whole existing radii changes approved shapes. A new lint or gate is
  excluded by the user's decision of 4 November 2026 to review radii.
- **Wider fix:** C. The shared package supplies the small-mark values to
  Astro, the React Native dashboard, and the mockups.
- **Previous examples:** The existing literals are:
  - `docs/mockups/src/components/figures.css:26`, the thin slot.
  - `docs/mockups/src/components/figures.css:48`, the large slot.
  - `docs/mockups/src/components/starts-chart.css:118`, a chart bar.
  - `docs/mockups/src/screens/marketing/cost.css:88`, a cost bar.
  - `docs/mockups/src/screens/savings/savings.css:60`, a savings bar.
  They were introduced in commits `aa5d2d9`, `661d1f8`, and `d790413`.
- **Recommendation:** C. It preserves every approved shape and prevents
  new consumers from copying the values or their arithmetic.
- **Action:** fix after merge, apply C, because the shared package and
  mockups are outside the site task. The direct site fix applies A.
