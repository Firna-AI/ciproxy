# Mockup styles reach into the icon markup

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch `calummoore/diagram-request-3jpv`,
  the check of review finding 5.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `docs/mockups/src/components/shell/app-shell.css:51` and
  `:66`, `docs/mockups/src/components/shell/account-menu.css:62`,
  `docs/mockups/src/components/shell/workspace-menu.css:76`,
  `docs/mockups/src/screens/auth/auth.css:75`, and
  `docs/mockups/src/screens/marketing/landing.css:169`.
- **Reason:** These rules were already on `main`, and the branch that found
  them changes only the docs figure.
- **Terms:**
  - **Lucide:** the icon set that the mockups render with `lucide-react`.
- **Background:**
  - The `typescript` skill says that code outside a component does not find
    or change its elements through selectors. It uses the props that the
    component offers.
  - `lucide-react` icons take a `className` prop.
  - The six rules above style Lucide icons with an `svg` element selector,
    such as `.side-item svg`. They set a colour, a margin, or
    `flex-shrink`.
  - The docs figure of How ciproxy works now passes a class through the
    icon prop instead, since review finding 5 of the branch above.
- **Problem:** The rules depend on the element that the icon renders.
- **Impact if no action:** An icon library change can drop these styles
  without an error.
- **Root question:** How do the mockup styles reach an icon through an
  interface that the icon offers?
- **Options:**
  - **A.** Pass a class to each icon through `className`, and style that
    class. Effort: Small.
  - **B.** Style the parent element, and let the icon inherit the colour
    through `currentColor`. A margin or `flex-shrink` still needs a class.
    Effort: Small.
- **Left out:** A lint for `svg` selectors, because the `typescript` skill
  rule already covers the class, and a new lint needs the user.
- **Wider fix:** None. The `typescript` skill rule is in force.
- **Previous examples:** Review finding 5 of the branch
  `calummoore/diagram-request-3jpv`.
- **Recommendation:** A. It follows the skill rule for every property.
- **Action:** fix after merge, apply A, because the rules are outside the
  diff of the branch that found them.
