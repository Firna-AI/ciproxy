# Dashboard API use

The [dashboard](../../README.md) reads and changes its data through the API.
The [dashboard guide](../../README.md#the-api) gives the base rules: one
origin, the generated types, and the one HTTP client with the session store.
This guide gives the rules of sign-in, the session, the install, and a new
API build.

## Sign-in and the session

- Sign-in is a top-level navigation to `/api/auth/github`. GitHub returns a
  sign-in to `/api/auth/github/callback` and an install to `/api/github/setup`.
  The API checks each return, then opens `/`, or `/sign-in?reason=github` on
  a failure. The app takes no installation ID from a URL.
- The session names the exact claims from the latest successful GitHub
  sign-in, including results kept after a failed authorization. These
  include installs that an owner approved later. When one
  named claim ended, `/` opens its workspace before a remembered one. When
  several ended, `/` opens the return page, which shows each outcome. A
  named claim that still waits adds no route, so the removed-App and
  ended-access pages stay first. The browser handles each ended
  continuation once, and claim order, time, and roles choose nothing.
- The account menu reads `can_claim_admin` directly from the session.
  The API returns false for a person who already has the Admin role.

## The install

- The session's install URL opens `/install` without creating an attempt.
  Each install action calls POST `/api/installation-links/start`, then opens
  its returned GitHub URL. The start preserves the session's install purpose.
  Add an organization uses its existing workspace action.
  A plain install action also works after the page observes a newer API build.

## A new API build

- Each API response names its build in the `x-ciproxy-build` header. After a
  new build, the next page change loads a new document, also through the
  back and forward buttons. Until then, the page keeps its input. A filter or
  range change after a new build loads a new document with the new address.

## Key Code

- [client.ts](client.ts) is the one HTTP client. The readers in `parse-*.ts`
  narrow each response body to its generated type;
  [parse-jobs.ts](parse-jobs.ts) reads one dashboard job, as a busy slot
  names it. [route-path.ts](route-path.ts) builds the path of a generated
  route.
- [navigation.ts](../state/navigation.ts) reloads on a new build.

## Related docs

- [Sign-in and access](../../../../docs/protocol/security.md#sign-in-and-access)
  gives the requirements of sign-in and the claims.
