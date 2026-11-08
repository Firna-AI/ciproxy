# Installation queries

These queries implement the installation association boundary of
[ciproxy-store-interface](../../../ciproxy-store-interface/README.md).
The [store crate](../../README.md) applies their forward migrations.

Attempts retain a state digest, requester identity, and explicit Direct/Add
evidence. Signed association joins the original requester and that person's own
pending or exact Add evidence under the installation transaction lock.
It replaces provisional other-user proof. It clears that user's offers,
target, and pending results.
The first signed association fixes Direct or Add. Replay and later browser proof
preserve that flow. Only signed association can replace provisional ownership.
Offers require both original-person proof and the signed event. Workspace
choice is immutable. Neither source workspace nor event order selects it.

Claim records hold no profile or credential. They do not keep a dashboard
person, session, or grant alive. An unlinked claim loses a deleted target
reference while preserving exact Add
provenance for safe recovery. Normal organization/workspace retention deletes
linked claims, their offers and pending results, and bound install attempts
with that registration.

Pending results hold only the claim ID, the exact requester ID, and a sequence
revision. A `Pending` claim save writes one in the claim transaction. Reads
return only rows whose claim still names that requester. Session establishment
deletes only the revisions that it read, so a later write survives.

Activation records distinguish admitted authority, billing hold, and current
Admin approval. Existing registrations are explicitly backfilled. Trusted
normal insertion initializes admission in its transaction. Missing metadata
cannot authorize a runner. Linked inactive organizations count against the
tier. Retained reinstalls preserve their policy rows.
The shared activation predicate controls preparation and registration.
Retained admission takes precedence over a fresh Add's initial holds.

Registration locks the workspace before counting organizations and writing
policy, resources, claim completion, and lifecycle output together. Billing
commits serialize on that same workspace row. Pending policy and installation
queries supply durable maintenance work after restart. Installation discovery
uses the interface's typed recovery state for signed Direct claims and verified
Add choices. Callback-only Direct claims still wait for their signed event.

Run checks from the repository root with DATABASE_URL set:

```sh
cargo test --package ciproxy-store-pg --test store_tests
cargo test --package ciproxy-api --test http_tests link_
```

- `claims.rs` saves token-free proof and preserves signed ownership and flow.
- `results.rs` records, reads, and consumes revisioned pending results.
- `association.rs` owns the locked signed identity and flow transaction.
- `offers.rs` stores authorized destinations and conditional choice.
- `activation.rs` commits independent admission and exact execution authority.
- `attempts.rs` preserves entry evidence independently of claim lifetime.
- `transaction.rs` preserves typed failures through atomic rollback.
