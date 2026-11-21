# Known npm advisories in the Expo 56 tools

- **Severity:** Low. **Effort:** Small. **Confidence:** High. **Harder
  later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-04, from the branch
  `conductor/ciproxy-mvp-foundation-m4-bootstrap`, pull request #2.
- **Location:** `ts/package-lock.json`, through `expo` and `expo-router` in
  `ts/app/package.json`.
- **Reason:** The dashboard has no users and no data yet, and one combined
  Expo upgrade with `futex-ai/accounting` and `@firna/ui` costs less.

- **Terms:**
  - **Expo:** the React Native framework that builds the ciproxy dashboard
    for the web, under `ts/app`.
  - **`@firna/ui`:** the shared UI library of the Firna products. It
    supports React Native versions below 0.86.
- **Background:** The dashboard app uses Expo 56, the same version as
  `futex-ai/accounting`. `npm audit --omit=dev` reports 16 advisories: 2
  low, 8 moderate, and 6 high. The high ones are in the Expo build tools
  and in `query-string` under `expo-router`.
- **Problem:** The build tools read only repository files. The
  `query-string` advisory lets a crafted link freeze the browser tab that
  opens it.
- **Impact if no action:** Low while the app has no users. Before launch,
  a known high advisory in a page dependency is a security finding.
- **Root question:** When does the dashboard get a dependency tree with no
  known high advisories?
- **Options:**
  - **A.** Keep Expo 56, and upgrade later together with
    `futex-ai/accounting` and `@firna/ui`. Effort: Small.
  - **B.** Upgrade to Expo 58 now. Its React Native version may fall
    outside the range that `@firna/ui` supports. Effort: Medium.
  - **C.** Keep Expo 56, and force fixed versions of the affected packages
    with npm `overrides`. This can break the Expo tools without a visible
    build error. Effort: Medium.
- **Left out:** Leaving it without a record, because a security advisory
  needs one.
- **Wider fix:** None. An audit check in the gate would be a new check,
  which needs the user's approval.
- **Previous examples:** `futex-ai/accounting` uses the same Expo versions.
- **Recommendation:** A, with the upgrade done before the paid launch.
- **Action:** fix after merge, apply A, because no user data is exposed
  today and one combined upgrade costs less.
