# The label list order differs from the mockups

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from the branch
  `calummoore/default-labels-rename`, review finding 1.
- **Location:** `crates/ciproxy-store-pg/src/labels.rs:71`,
  `ts/app/src/view/settings-labels.ts:101`,
  `docs/mockups/src/screens/settings/labels.tsx:70`,
  `docs/mockups/src/fixtures/settings.ts:39`, and
  `ts/app/scripts/m20-fixtures-labels.ts:180`.
- **Reason:** No requirement gives the order of the label list, and the
  dashboard and its mockups disagree, so the user chooses the order.

- **Terms:**
  - **Label:** the value in `runs-on` that sends a job to ciproxy and
    picks the platform and the size of its VM. `docs/protocol/labels.md`
    gives the rules.
  - **Default labels:** the five labels of each new workspace: `ciproxy`,
    `ciproxy-medium`, `ciproxy-large`, `ciproxy-macos`, and
    `ciproxy-macos-medium`.
  - **Mockup:** a sample screen under `docs/mockups`. The mockups are part
    of the spec, and the `mockups` skill keeps them aligned with the
    product.
  - **Browser fixture:** the sample API data under `ts/app/scripts/` that
    the browser checks of the dashboard use instead of the live API.
- **Background:**
  - The Labels tab of Settings in the dashboard lists the labels of a
    workspace. The list comes from `GET /workspaces/{workspace_id}/labels`.
  - The store reads the labels in name order, with
    `order(labels::name)` in `crates/ciproxy-store-pg/src/labels.rs`. The
    API and the dashboard keep that order. Nothing else sorts the list.
  - On 22 November 2026, the user renamed the default labels
    `ciproxy-4vcpu`, `ciproxy-8vcpu`, and `ciproxy-macos-4vcpu` to
    `ciproxy-medium`, `ciproxy-large`, and `ciproxy-macos-medium`. The old
    names sorted by size by chance, because digits sort before letters.
  - A PostgreSQL 15 query with the collation C gives this order for the
    new names: `ciproxy`, `ciproxy-large`, `ciproxy-macos`,
    `ciproxy-macos-medium`, `ciproxy-medium`.
  - Every mockup state shows the Linux labels first, from the smallest
    size to the largest, and then the macOS labels. A custom label comes
    after the default labels of its platform.
  - No doc under `docs/`, no README, and no plan gives a rule for the
    order. No browser check asserts the order of the rows.
- **Problem:**
  - A new workspace lists `ciproxy-large` before `ciproxy-medium`, and
    the macOS labels sit between the Linux labels.
  - The mockups show `ciproxy-medium` before `ciproxy-large`, and the
    macOS labels last.
  - `main` already had the same mismatch for custom labels. The
    nothing-fits and label-waits states of
    `docs/mockups/src/screens/settings/labels.tsx` show `ciproxy-32vcpu`
    last, but the name order puts it second.
- **Impact if no action:** The Labels tab of each new workspace shows a
  mixed order that differs from the mockups. Customers can still find and
  edit each label.
- **Root question:** What order should customers see in the Settings label
  list?
- **Options:**
  - **A.** Move `ciproxy-large` before `ciproxy-medium` in the mockups and
    in the browser fixture, so that they follow the name order. In the
    macOS states, also move `ciproxy-medium` below the two macOS labels. A
    later rename needs the same manual check. Effort: Small.
  - **B.** Sort the mockup lists and the browser fixture replies by name,
    and keep the order of the product. Sort in `labelsOf` of
    `docs/mockups/src/screens/settings/labels.tsx`, not in the shared
    fixture, because the docs mockup `labels-reference.tsx` keeps the
    size order of the site page. A JavaScript sort matches the store only
    for lowercase names, because collations sort capital letters in
    different ways. Effort: Small.
  - **C.** Specify that the default labels appear in the order of their
    starting size. Apply that order in the dashboard and in both sets of
    examples. Define where custom labels and older labels appear. Effort:
    Medium.
  - **D.** Sort the list in the dashboard by one rule for every label.
    - Linux labels come first, then macOS labels. Each group goes from the
      smallest size to the largest, by vCPU, then memory, then free disk,
      then name.
    - Write the rule in `docs/protocol/labels.md`.
    - Sort in `labelRows` of `ts/app/src/view/settings-labels.ts`, and add
      a case to `ts/app/_tests_/settings-labels.test.ts`.
    - Sort the mockup list by the same keys. Every mockup state already
      shows this order, and the browser fixture needs no change.
    - Cost: a new requirement, and a size edit moves a label. A custom
      label that is smaller than `ciproxy` sorts above it, unless the rule
      keeps `ciproxy` first.
    - Effort: Medium.
  - **E.** Show the labels in the order that the workspace made them.
    - The default labels come first, in the order of the table in
      `docs/protocol/labels.md`. Each custom label follows in the order
      that it was added.
    - The `labels` table records no creation time, and the label IDs are
      random. So E needs a migration that adds a sequence column with a
      backfill, a sort in `crates/ciproxy-store-pg/src/labels.rs`, a store
      test, and a requirement.
    - Cost: a migration. The custom labels of the two platforms mix. A
      safe rename, which adds a new label and deletes the old one, moves
      the label to the end.
    - Effort: Medium.
- **Left out:**
  - Keep both orders: it fails the goal, because the mockups must stay
    aligned with the product.
  - Default names that sort into size order by name: they break the names
    that the user chose on 22 November 2026.
  - An order that the customer sets, such as a drag control: it fails the
    goal alone, because each new workspace and each new label still need
    a first order.
  - A sort in the API: not a separate choice. The Labels tab is the only
    full list of labels, and the browser fixture would have to copy the
    rule.
- **Wider fix:** D. It takes the order from the platform and the size, so
  a rename, the order of fixture data, and the database collation cannot
  change it. The first pass of the review named B.
- **Previous examples:** Commit `1733ed3` fixed a similar mismatch between
  the docs mockups and the rendered site.
- **Recommendation:** D. It gives the size order that every mockup state
  and the docs tables show, with no migration. The first pass of the
  review recommended B, which would put the mixed order into the macOS
  states of the spec.
- **Action:** ask the user, because the dashboard and its mockups disagree
  and no requirement says which order to keep.
