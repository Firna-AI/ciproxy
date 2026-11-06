# @ciproxy/api

The generated dashboard HTTP contract. Rust types in
[ciproxy-api](../../crates/ciproxy-api/README.md) own the requests, responses,
errors, paths, and query parameters. Dashboard code imports these types. It does
not define another backend data model.

## Commands

Run from the repository root:

```sh
npm --prefix ts run api:generate
npm --prefix ts run api:generate:check
npm --prefix ts run api:typecheck
npm --prefix ts run api:lint
```

Generation requires the pinned Rust and Node toolchains. `utoipa` emits OpenAPI
3.1 from the same route registration that builds the axum router.
`openapi-typescript` reads that document and writes the declaration file. The
generated directory contains both files. Commit both files together. Never edit
them by hand.

The approved `api-types` gate phase regenerates the contract in `.context` and
compares it with the committed output. The normal TypeScript gate checks this
package with strict TypeScript. It checks generated declarations too. Generated
output keeps the generator's own format.

## Use

```ts
import type { components, paths } from "@ciproxy/api";

type Session = components["schemas"]["SessionView"];
type SessionRoute = paths["/session"];
```

IDs use strings. Money uses integer US cents as decimal strings. Times use UTC
RFC 3339. Byte counts and other large counters use decimal strings.

## Key Code

- [generate.ts](generate.ts) runs the Rust generator and checks drift.
- [generated/openapi.json](generated/openapi.json) is the API reference.
- [generated/api.d.ts](generated/api.d.ts) holds generated paths and schemas.
- [index.ts](index.ts) is the package's type entry point.
