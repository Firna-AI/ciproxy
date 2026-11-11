import assert from "node:assert/strict";
import test from "node:test";

import { parseOverflowSummary } from "../src/api/machine-parse.ts";
import { overflowModel } from "../src/view/machines-overflow.ts";
import { nbsp, summary } from "./machines-fixtures.ts";

test("a connected Boat account shows its wait and the confirmed spend against the limit", () => {
  assert.deepEqual(overflowModel(summary()), {
    caption: `Connected to the account mokly${nbsp}· jobs start there after 30${nbsp}seconds`,
    meter: { ratio: 0.306, value: "$61.20 of $200", detail: null },
    note: null,
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
    pending.caption,
    `Connected to the account mokly${nbsp}· jobs start there at once`,
  );
  assert.equal(pending.meter?.detail, "$4.10 still pending");
});

test("Boat that takes no jobs says why, and jobs run on the machines only", () => {
  assert.equal(
    overflowModel(summary({ account: null })).caption,
    "Boat is not connected. Jobs run on your machines only.",
  );
  assert.equal(
    overflowModel(summary({ available: false })).caption,
    "Jobs run on your machines only.",
  );
  assert.equal(
    overflowModel(summary({ enabled: false })).caption,
    "Overflow is off, so jobs run on your machines only.",
  );
  const rejected = summary({
    account: {
      connected_at: "2026-09-15T09:00:00Z",
      name: "mokly",
      rejected_at: "2026-11-12T08:00:00Z",
    },
  });
  assert.equal(
    overflowModel(rejected).caption,
    `Boat rejected the API key on 12${nbsp}November. Jobs run on your machines only until you connect Boat with a new key.`,
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
    overflowModel(limit).caption,
    `Boat spend reached the $200 limit. Jobs run on your machines only. Boat takes jobs again on 1${nbsp}December.`,
  );
  assert.deepEqual(overflowModel(summary({ spend: null })), {
    caption: `Connected to the account mokly${nbsp}· jobs start there after 30${nbsp}seconds`,
    meter: null,
    note: "Boat spend shows here when Boat reports it.",
  });
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
