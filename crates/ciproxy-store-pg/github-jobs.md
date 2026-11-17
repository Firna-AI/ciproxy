# GitHub job store

This guide gives the PostgreSQL queries for GitHub scale-set job demand,
its retention, and the Actions timing writes. The [crate README](README.md)
gives the rest of the store.

## Demand and delivery revisions

UUID scale-set demand stays separate from numeric Actions job history.
Each demand write saves a pending delivery revision in the same statement.
Stale replay cannot replace a later assignment or a terminal state.
An exact UTC text key preserves nanosecond assignment order across the
microsecond precision of PostgreSQL timestamps.
Delivery confirmation cannot consume a newer revision.

## Retention

The GitHub retention query deletes completed demand after 90 days,
including pending deliveries. Active work stays.
When GitHub omitted its completion time, the first trusted observation
supplied by the caller starts the period. Exact replay keeps that instant.
The query stores this observation separately from GitHub's metadata.

## Actions timing writes

Authoritative Actions timing writes change only the times, the completion
phase, the numeric Actions job ID, and a missing result. GitHub's
conclusion fills a missing result. A saved non-null result stays, in both
the current demand and the retained attempt. A time or a result that
GitHub does not supply keeps its saved value, or stays absent. The phase
never moves backwards.

Each write locks the current demand row first. One transaction then saves
the retained assignment, its matching current demand, and the pending
delivery revision. Each write patches the saved rows in place, so it
preserves listener metadata when writes overlap. An older assignment cannot
change the current demand. Duplicate timing keeps the revision unchanged.
The first completion observation stays unchanged.

[Completion recovery](../ciproxy-github/run-metadata.md#job-completion-recovery)
in the GitHub integration says which attempts the pending timing query
returns.
