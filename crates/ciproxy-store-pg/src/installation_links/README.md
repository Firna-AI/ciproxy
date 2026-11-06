# Installation queries

These queries implement the installation association boundary of
[ciproxy-store-interface](../../../ciproxy-store-interface/README.md).
The [store crate](../../README.md) applies their forward migrations.

Attempts retain a state digest, requester identity, and explicit Direct/Add
evidence. Claims merge fresh-user proof with signed events under an exact
installation transaction lock. A later signed original requester replaces
provisional other-user proof and clears that user's offers and target.
Offers require both original-person proof and the signed event. Workspace
choice is immutable. Neither source workspace nor event order selects it.

Claim records hold no profile or credential. They do not keep a dashboard
person, session, or grant alive. An unlinked claim loses a deleted target
reference while preserving exact Add
provenance for safe recovery. Normal organization/workspace retention deletes
linked claims, their offers, and bound install attempts with that registration.

Activation records distinguish admitted authority, billing hold, and current
Admin approval. Existing registrations are explicitly backfilled. Trusted
normal insertion initializes admission in its transaction. Missing metadata
cannot authorize a runner. Linked inactive organizations count against the
tier. Retained reinstalls preserve their policy rows.

Registration locks the workspace before counting organizations and writing
policy, resources, claim completion, and lifecycle output together. Billing
commits serialize on that same workspace row. Pending policy and verified
choice queries supply durable maintenance work after restart.

Run checks from the repository root with DATABASE_URL set:

```sh
cargo test --package ciproxy-store-pg --test store_tests
cargo test --package ciproxy-api --test http_tests link_
```

- `claims.rs` joins exact signed events and token-free proof.
- `offers.rs` stores authorized destinations and conditional choice.
- `activation.rs` commits independent admission and exact execution authority.
- `attempts.rs` preserves entry evidence independently of claim lifetime.
- `transaction.rs` preserves typed failures through atomic rollback.
