# Repository gate omits site calculation tests

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-04, from
  `conductor/ciproxy-mvp-milestone-23-site`.
- **Location:** `ts/package.json`, `ts/site/package.json`,
  `ts/site/_tests_/pricing.test.ts`, and `xtask/src/check/phases.rs`.
- **Reason:** The site task cannot change the root TypeScript scripts or
  the repository gate.

- **Terms:**
  - **Gate:** `cargo xtask check`, the repository checks that also run in
    GitHub Actions.
- **Background:** The site has Node tests for its cheapest-plan choice,
  the cost claim, copied workflow examples, and search state. They run with
  `npm --prefix ts --workspace @ciproxy/site run test`. The gate runs site
  lint, typecheck, and build, but does not run that test script. The site
  task runs these tests manually and proves that wrong calculations fail.
  The browser focus tests under `ts/site/_tests_/browser/` run with
  `npm --prefix ts --workspace @ciproxy/site run test:browser`. They build
  the production site and need Google Chrome. CI runs only the gate, so
  these browser tests also run manually. Browser CI needs a separate
  approved gate change.
- **Task:** Connect the existing site test script to the gate's existing
  TypeScript phase. Add the matching root package script and document it
  beside the existing commands. Keep this a run of the existing tests.
- **Impact if no action:** A change can pass CI without running the tests
  of the published cost claim and cheapest-plan calculation.
- **Needs:** A change that owns `ts/package.json`, `xtask/`, and their
  command documentation.
- **Done when:** The complete gate runs the existing site Node tests. A
  wrong result in either calculation fails its assertion in that gate run. The
  restored code passes the complete gate.
