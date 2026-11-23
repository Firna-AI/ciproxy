import assert from "node:assert/strict";
import test from "node:test";

import { parseOverflowSummary } from "../src/api/machine-parse.ts";
import type { Problem } from "../src/api/schema.ts";
import { machinesServices } from "../src/state/machines-services.ts";
import { overflowModel } from "../src/view/machines-overflow.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import { nbsp, summary } from "./machines-fixtures.ts";
import { appWith, response } from "./support.ts";

const steadySpend = {
  name: "Boat",
  confirmed: "$61.20",
  unconfirmed: "$0",
  settled: null,
  limit: "$200",
  confirmedShare: 0.306,
  unconfirmedShare: 0,
  reached: false,
};

test("each connected provider has a row with its account and its spend, and the wait shows once under the rows", () => {
  assert.deepEqual(overflowModel(summary()), {
    kind: "providers",
    rows: [
      {
        provider: "boat",
        name: "Boat",
        account: "Connected to the account mokly",
        hold: null,
        spend: steadySpend,
        note: null,
      },
    ],
    caption: `Jobs wait 30${nbsp}seconds for a free slot on your machines, then go to the next place of their label.`,
  });
  const pending = overflowModel(
    summary({
      wait_seconds: 0,
      spend: {
        bounded: true,
        confirmed_cents: "6120",
        limit_reached: false,
        settled_bound_cents: "0",
        unconfirmed_cents: "410",
      },
    }),
  );
  assert.equal(
    pending.kind === "providers" ? pending.caption : null,
    "When your machines are full, jobs go to the next place of their label at once.",
  );
  assert.equal(
    pending.kind === "providers" ? pending.rows[0]?.spend?.unconfirmed : null,
    "$4.10",
  );
});

test("a provider that takes no jobs says why, and no connected provider leaves the machines only", () => {
  const holdOf = (model: ReturnType<typeof overflowModel>) =>
    model.kind === "providers" ? model.rows[0]?.hold : "no rows";
  assert.deepEqual(overflowModel(summary({ account: null })), {
    kind: "none",
  });
  assert.equal(
    holdOf(overflowModel(summary({ available: false }))),
    "Boat is unavailable right now, so jobs skip it.",
  );
  const off = overflowModel(summary({ enabled: false }));
  assert.equal(
    off.kind === "providers" ? off.caption : null,
    "Overflow providers are off, so jobs run on your machines only.",
  );
  assert.equal(holdOf(off), null);
  const rejected = summary({
    account: {
      connected_at: "2026-09-15T09:00:00Z",
      name: "mokly",
      rejected_at: "2026-11-12T08:00:00Z",
    },
  });
  assert.equal(
    holdOf(overflowModel(rejected)),
    `Boat rejected the API key on 12${nbsp}November. ciproxy starts no new jobs on Boat until you connect a new key.`,
  );
  const limit = summary({
    spend: {
      bounded: true,
      confirmed_cents: "20000",
      limit_reached: true,
      settled_bound_cents: "0",
      unconfirmed_cents: "0",
    },
  });
  assert.equal(
    holdOf(overflowModel(limit)),
    `Boat spend reached the $200 limit. ciproxy starts no new jobs on Boat until 1${nbsp}December.`,
  );
  const unread = overflowModel(summary({ spend: null }));
  assert.deepEqual(
    unread.kind === "providers"
      ? [unread.rows[0]?.spend, unread.rows[0]?.note]
      : null,
    [null, "Boat spend shows here when Boat reports it."],
  );
});

test("the overflow summary reader keeps nulls and refuses a negative wait", () => {
  const body = JSON.parse(JSON.stringify(summary({ spend_limit_cents: null })));
  assert.deepEqual(
    parseOverflowSummary(body),
    summary({ spend_limit_cents: null }),
  );
  assert.equal(parseOverflowSummary({ ...body, wait_seconds: -1 }), undefined);
  assert.equal(parseOverflowSummary({ ...body, enabled: "yes" }), undefined);
});

test("a failed refresh keeps the last overflow summary and shows the failure until a read works", async () => {
  const unavailable = {
    code: "unavailable",
    message: "Boat did not answer.",
  } satisfies Problem;
  const later = summary({ wait_seconds: 0 });
  const { app, calls } = appWith(
    response(summary()),
    response(unavailable, 503),
    response(later),
  );
  const store = machinesServices(app).overflow(workspaceId);
  await store.load();
  await store.load();
  assert.deepEqual(store.state.get(), {
    status: "ready",
    page: summary(),
    refreshFailure: { kind: "problem", status: 503, problem: unavailable },
  });
  await store.load();
  assert.deepEqual(store.state.get(), {
    status: "ready",
    page: later,
    refreshFailure: null,
  });
  assert.deepEqual(
    calls.map((call) => call.url),
    Array(3).fill(`/api/workspaces/${workspaceId}/overflow/summary`),
  );
});
