# Banner and Savings query names skip generated types

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  pull request #53, two new problems that the check of owner finding 5
  found.
- **Source:** Review.
- **Category:** Repository rule.
- **Location:** `ts/app/src/api/banner.ts:19`,
  `ts/app/src/api/savings.ts:17`, and `ts/app/src/api/route-path.ts:30`.
- **Reason:** The owner's fix round of the branch that found the problem
  typed only the review query, and it took no other cleanup.

- **Terms:**
  - **`withQuery`:** the helper at `ts/app/src/api/route-path.ts:30-40` that
    adds a query to a path. It takes any record of strings, so it checks no
    query name.
  - **Generated operation query:** the type
    `operations["<operation>"]["parameters"]["query"]` of
    `ts/api/generated/api.d.ts`. `openapi-typescript` generates it from the
    Rust query type.
- **Background:**
  - The shared-types rule says that Rust owns every type that crosses the
    boundary, also the query parameters. The frontend imports the generated
    type and writes none by hand. `ts/api/README.md:3-6` says that the Rust
    types own the query parameters, and that dashboard code imports them.
  - The banner read, at `ts/app/src/api/banner.ts:13-19`, sends
    `{ page: scope === "organizations" ? "organizations" : undefined }`.
    The generated `page_banner` query is `{ page?: "organizations" }`. No
    type ties the name or the value to it. Only the Organizations scope
    sends the name, and no page asks for that scope yet.
  - The Savings read, at `ts/app/src/api/savings.ts:11-18`, sends
    `{ range }`. The generated `savings` query is
    `{ range?: SavingsRange }`. The value type `SavingsRange` is generated,
    but no type ties the name `range` to the operation.
  - Jobs history already follows the rule. `HistoryQuery`, at
    `ts/app/src/api/jobs-types.ts:25-28`, is
    `NonNullable<operations["jobs_history"]["parameters"]["query"]>`, and
    `ts/app/src/api/jobs.ts:20-28` gives it to `withQuery`.
  - The review query follows it since the owner's fix of 2026-11-08.
    `ReviewRequest`, at `ts/app/src/api/billing.ts:19-22`, applies
    `Required` and `NonNullable` to
    `operations["checkout_review"]["parameters"]["query"]`, and its query
    object `satisfies Record<keyof ReviewRequest, string>`. The owner chose
    this direct fix for the review query only.
  - The Rust handlers read each query with the axum `Query` extractor. No
    query type in `crates/ciproxy-api/src` sets `deny_unknown_fields`. So
    the server ignores a name that it does not know, and it uses its
    default. The banner then gets the page scope, at
    `crates/ciproxy-api/src/handlers/billing_pages.rs:113`. Savings gets the
    last 30 days, at `crates/ciproxy-api/src/handlers/savings.rs:25` and
    `crates/ciproxy-api/src/wire/savings.rs:16-17`.
  - Exact-URL tests guard the requests today, at
    `ts/app/_tests_/billing-review.test.ts:74`,
    `ts/app/_tests_/jobs-api.test.ts:43-50`, and
    `ts/app/_tests_/billing-banner.test.ts:144` and `:296`.
- **Problem:** The banner and Savings reads send query names that no
  generated operation type checks, against the shared-types rule.
- **Impact if no action:** After a Rust rename of `page` or `range`, the app
  still compiles and sends the old name. The server ignores it, so the
  Organizations tab gets the page banner, and Savings gets the report of the
  last 30 days.
- **Root question:** How does every query that the dashboard sends follow
  the Rust contract?
- **Options:**
  - **A.** Derive the query type of each read from its generated operation,
    as `HistoryQuery` does, and check each query object against it with
    `satisfies`. Change only `banner.ts` and `savings.ts`, with no runtime
    change. Cost: each new query read must repeat the pattern. Effort:
    Small.
  - **B.** Type `withQuery` by route. It takes the route, its path values,
    and a query typed from the generated operation of that route. It turns
    each number into a decimal string in one place. The review keeps
    `ReviewRequest` and its `Required` values at its call site. Then every
    query read fails the type check when its Rust query changes: the review,
    Jobs history, Savings, and the banner. Cost: the shared helper and its
    four call sites change, and the exact-URL tests guard the runtime.
    Effort: Small.
- **Left out:**
  - No change, because the shared-types rule and `ts/api/README.md:3-6`
    require the generated type.
  - Another hand-written interface or schema, because it copies the
    contract again.
  - A generated HTTP client, such as `openapi-fetch`, because it adds a
    dependency and replaces the one HTTP client that every read uses, as
    `ts/app/README.md` says.
  - `deny_unknown_fields` or required values in the Rust query types,
    because B catches the same drift in the type check with no API change.
    They also add a new API rejection, which needs the user.
  - A lint or a test for the class, because B prevents the same drift in
    the helper that every read uses, with no new check. A new check also
    needs the user.
- **Wider fix:** B. One typed helper makes every query read follow its
  generated operation, also each new read.
- **Previous examples:** The review query in `ts/app/src/api/billing.ts` was
  typed by hand until the owner's fix of 2026-11-08 derived it from
  `checkout_review`.
- **Related:** `plans/follow-ups/inactive-banner-hold-reasons.md`, same read:
  it words the banner that the `page` name of this item asks for. Neither
  item waits for the other.
- **Recommendation:** B. One typed helper covers every query read, also each
  new one, and it keeps the owner's fix of the review query. A is the
  smallest change.
- **Action:** fix after merge, apply B, because the owner's fix round of the
  branch that found the problem typed only the review query and took no
  other cleanup.
