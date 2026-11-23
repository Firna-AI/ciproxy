# Smoke commands read the API by hand

- **Severity:** Medium. **Effort:** Large. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/ciproxy-providers-interface-s8rp`, review finding 5, the
  wider fix.
- **Location:** `crates/ciproxy-control-plane/scripts/overflow_smoke/`,
  `crates/ciproxy-control-plane/scripts/overflow-smoke.py`, and
  `crates/ciproxy-control-plane/scripts/binding-smoke.py`.
- **Reason:** The wider fix deletes the Python commands or adds a gate step,
  and both need the user.

- **Source:** Review.
- **Category:** Test.
- **Terms:**
  - **Live smoke command:** an operator command that runs real GitHub jobs
    against a deployed control plane, to check routing or registration.
- **Background:**
  - The overflow and binding smoke commands are Python. They read and write
    the dashboard API through plain dictionaries.
  - The provider-interface change removed fields that they read, such as
    the single Boat spend limit and the label switches. The commands then
    failed with `KeyError` before their scenarios ran.
  - Review finding 5 moved the commands to the new fields by hand.
  - The dashboard app already has typed API endpoints and parsers in
    `ts/app/src/api/`. `ts/app/tsconfig.json` includes `scripts/**/*.ts`,
    so the `api-types` and `typescript` gate phases fail on a removed field
    in a TypeScript script.
- **Problem:** No check fails when the API drops a field that the Python
  commands read, so the next API change can break them again unseen.
- **Impact if no action:** An operator finds the break only when a live
  smoke run fails, which costs a deployment and a GitHub job each time.
- **Root question:** How do the operator commands learn at the gate that the
  API changed under them?
- **Options:**
  - **B.** Generate a Python client from the OpenAPI document, and add a gate
    step that regenerates it and type-checks the scripts. Effort: Medium.
  - **C.** Port both commands to TypeScript under `ts/app/scripts/`, beside
    `boat-live-check.ts`, and use the typed endpoints and parsers. Keep the
    GitHub, Boat, and `psql` steps. The gate phases that run today then
    catch a removed field or route. This deletes the Python commands and
    rewrites about 750 lines and their local tests. Effort: Large.
- **Left out:** A list of the fields that the scripts read, because it must
  know each field. A server that rejects unknown fields, because it adds a
  stricter validation for every client and does not make the commands run.
- **Wider fix:** C. It needs no new gate step.
- **Previous examples:** `plans/follow-ups/harness-reauthorization-url-drift.md`
  records a smoke harness that kept an older response shape.
- **Recommendation:** C. The existing gate then covers the commands.
- **Action:** ask the user, because C deletes the Python commands, and B adds
  a gate step.
