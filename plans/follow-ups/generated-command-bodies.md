# API change bodies skip their generated route types

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  pull request #53, the wider fix that the check of owner finding 1 split
  off.
- **Source:** Review.
- **Category:** Repository rule.
- **Location:** `ts/app/src/api/client.ts:30-35`,
  `ts/app/src/api/endpoints.ts:29-72`, `ts/app/src/api/billing.ts:63-92`,
  and `ts/app/src/api/billing-payment.ts:83-114`.
- **Reason:** The integration step of the branch that found the problem
  typed only the paid change body, and the client change reaches the base
  client and all 13 of its write calls.

- **Terms:**
  - **`ApiClient.change`:** the write method of the one HTTP client, at
    `ts/app/src/api/client.ts:30-35`. It takes a method, a filled path, a
    parser, and `body?: unknown`.
  - **Generated operation body:** the type
    `operations["<operation>"]["requestBody"]["content"]["application/json"]`
    of `ts/api/generated/api.d.ts`. `openapi-typescript` generates it from
    the Rust request type.
  - **`routePath`:** the helper at `ts/app/src/api/route-path.ts:16-24`. It
    fills a route of the generated paths, so a renamed route fails the type
    check.
- **Background:**
  - The shared-types rule says that Rust owns every type that crosses the
    boundary. `ts/api/README.md:3-6` says that the Rust types own the
    requests, and that dashboard code imports them.
  - Every write of the dashboard goes through `ApiClient.change`. It has 13
    calls: 7 in `ts/app/src/api/endpoints.ts`, 3 in
    `ts/app/src/api/billing.ts`, and 3 in `ts/app/src/api/billing-payment.ts`.
    Five of them send a body.
  - Four bodies use aliases of generated schemas: `WorkspaceChoice`,
    `CheckoutReturn`, `PlanAddOns`, and `PlanSelection`. Since the
    integration of 8 November 2026, the paid change body
    `PaidChangePayment` comes from the generated operation `billing_change`,
    at `ts/app/src/api/billing-types.ts`.
  - No type ties a body to the operation of its route and method. The
    client accepts any value for any path.
  - The axum JSON extractor refuses a body that misses a field. The router
    sends that as HTTP 422 with the code `invalid_request`, at
    `crates/ciproxy-api/src/router.rs:142-150`.
- **Problem:** A write body is not tied to the generated operation of its
  route and method. After a Rust change of a request body, the app still
  compiles and sends the old body.
- **Impact if no action:** The server refuses the old body with 422
  `invalid_request`, so that action fails for every user. A test that pins
  the exact old body still passes.
- **Root question:** How does every request body that the dashboard sends
  follow the Rust contract?
- **Options:**
  - **A.** Derive the body type of each write from the generated operation
    of its route, as the paid change body does. Change only the write
    modules, not the client. Cost: each new write must repeat the pattern,
    and the client still accepts a body of any type. Effort: Small.
  - **B.** Type `ApiClient.change` by route and method. It takes the route,
    its path values, and a body typed from the generated operation of that
    route and method, and it builds the path with `routePath`. Then each
    write fails the type check when its Rust request body changes, also
    each new write. Cost: the base client, its test doubles, and its 13
    calls change, and the exact-body tests guard the runtime. Effort:
    Medium.
- **Left out:**
  - No change, because the shared-types rule and `ts/api/README.md:3-6`
    require the generated type.
  - Another hand-written interface or schema, because it copies the
    contract again.
  - A generated HTTP client, such as `openapi-fetch`, because it adds a
    dependency and replaces the one HTTP client that every read and write
    uses.
  - A runtime check of each body, a lint, or a test for the class, because
    B prevents the same drift in the type check with no new check. A new
    check also needs the user.
- **Wider fix:** B. One typed client makes every write body follow its
  generated operation, also each new write.
- **Previous examples:** Pull request #39 changed the body of
  `billing_change` from a `PlanSelection` to `PaidChangePayment`, which
  holds only `quote_id`. After the merge, the app still compiled and sent
  the selection. The exact-body test at
  `ts/app/_tests_/billing-review.test.ts:104-108` of commit `160f311`
  expected the selection, so it passed too. Every move between paid plans
  then failed with 422. The integration of 8 November 2026 typed that one
  body from the generated operation.
- **Related:** `plans/follow-ups/generated-query-names.md`, same boundary:
  it types the query names of reads, and this item the bodies of writes.
  One route-typed helper can serve both, but neither item waits for the
  other.
- **Recommendation:** B. One typed client covers every write body, also
  each new one. A is the smallest change.
- **Action:** fix after merge, apply B, because it reaches the base client
  and all 13 of its write calls, outside the integration step that typed
  only the paid change body.
