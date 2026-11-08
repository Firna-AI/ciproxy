import assert from "node:assert/strict";
import test from "node:test";

import type { ApiFailure } from "../src/api/client.ts";
import type { ProblemCode } from "../src/api/schema.ts";
import {
  boatPanelModel,
  boatProgress,
  connectedSummary,
  overflowNotice,
  spendFigures,
  waitOptions,
} from "../src/view/boat.ts";
import {
  connectProblem,
  keyRefused,
  limitSaveProblem,
} from "../src/view/boat-failures.ts";
import {
  boatSpend,
  notConnected,
  overflowSettings,
} from "../scripts/fixtures-boat.ts";

// A zone west of UTC shows a date that a local-time format would move.
process.env.TZ = "America/Los_Angeles";

const rejected = (at: string) => {
  const settings = overflowSettings();
  const account = settings.account ?? null;
  return { ...settings, account: account && { ...account, rejected_at: at } };
};

function problem(status: number, code: ProblemCode): ApiFailure {
  return { kind: "problem", status, problem: { code, message: "API copy" } };
}

test("the spend shows confirmed and unconfirmed parts, and only the API flag reaches the limit", () => {
  assert.deepEqual(spendFigures(overflowSettings()), {
    confirmed: "$48.10",
    unconfirmed: "$13.10",
    settled: "$1.20",
    limit: "$200",
    confirmedShare: 0.2405,
    unconfirmedShare: 0.0655,
    reached: false,
  });
  const rounded = overflowSettings({
    spend: boatSpend({ confirmed_cents: "15000", unconfirmed_cents: "5000" }),
  });
  assert.equal(spendFigures(rounded)?.reached, false);
  const flagged = overflowSettings({
    spend: boatSpend({ limit_reached: true }),
  });
  assert.equal(spendFigures(flagged)?.reached, true);
  const unknown = overflowSettings({
    spend: boatSpend({ bounded: false, limit_reached: null }),
  });
  assert.equal(spendFigures(unknown)?.reached, false);
  assert.equal(
    spendFigures(
      overflowSettings({ spend: boatSpend({ settled_bound_cents: "0" }) }),
    )?.settled,
    null,
  );
  assert.equal(spendFigures(overflowSettings({ spend: null })), null);
});

test("the page notice names a rejected key, a reached limit with its UTC budget month and resume day, or unknown costs", () => {
  assert.equal(overflowNotice(overflowSettings()), null);
  assert.deepEqual(overflowNotice(rejected("2026-11-12T20:00:00Z")), {
    tone: "danger",
    title: "Boat rejected the API key on 12\u00a0November",
    body: "Jobs run on your machines only until you connect Boat with a new key.",
  });
  const reached = overflowSettings({
    spend: boatSpend({ limit_reached: true }),
  });
  assert.deepEqual(overflowNotice(reached), {
    tone: "warning",
    title: "Boat spend reached the $200 limit for November",
    body: "Jobs run on your machines only, and wait there for a free slot. Boat takes jobs again on 1\u00a0December, or when you raise the limit.",
  });
  assert.deepEqual(
    overflowNotice({ ...reached, next_month_at: "2027-01-01T00:00:00Z" }),
    {
      tone: "warning",
      title: "Boat spend reached the $200 limit for December",
      body: "Jobs run on your machines only, and wait there for a free slot. Boat takes jobs again on 1\u00a0January, or when you raise the limit.",
    },
  );
  assert.equal(
    overflowNotice({
      ...rejected("2026-11-12T20:00:00Z"),
      spend: reached.spend,
    })?.tone,
    "danger",
  );
  assert.deepEqual(
    overflowNotice(
      overflowSettings({
        spend: boatSpend({ bounded: false, limit_reached: null }),
      }),
    ),
    {
      tone: "warning",
      title: "Boat costs are not known yet",
      body: "Jobs run on your machines only until Boat reports the cost of each start.",
    },
  );
});

test("the wait offers the five waits of job routing and keeps another saved wait", () => {
  assert.deepEqual(
    waitOptions(30).map((option) => [option.value, option.label]),
    [
      ["0", "At once"],
      ["15", "15 seconds"],
      ["30", "30 seconds"],
      ["60", "1 minute"],
      ["120", "2 minutes"],
    ],
  );
  assert.deepEqual(
    waitOptions(45).map((option) => option.label),
    [
      "At once",
      "15 seconds",
      "30 seconds",
      "45 seconds",
      "1 minute",
      "2 minutes",
    ],
  );
});

test("Setup counts Boat as connected only with an accepted key, and a skip only for its person", () => {
  const ready = { status: "ready" as const, refreshFailure: null };
  assert.equal(
    connectedSummary(overflowSettings()),
    "Connected to the Boat account northstar · spend limit $200 a month",
  );
  assert.equal(connectedSummary(rejected("2026-11-12T20:00:00Z")), null);
  assert.deepEqual(
    boatProgress({ ...ready, settings: overflowSettings() }, "mira", "mira"),
    {
      connected:
        "Connected to the Boat account northstar · spend limit $200 a month",
      skipped: false,
    },
  );
  const none = { ...ready, settings: overflowSettings({ account: null }) };
  assert.deepEqual(boatProgress(none, "mira", "mira"), {
    connected: null,
    skipped: true,
  });
  assert.deepEqual(boatProgress(none, "mira", "sasha"), {
    connected: null,
    skipped: false,
  });
  assert.deepEqual(boatProgress({ status: "loading" }, null, "mira"), {
    connected: null,
    skipped: false,
  });
});

test("a refused key belongs to the key field, a refused limit to the limit field", () => {
  assert.deepEqual(connectProblem(problem(422, "boat_key_rejected")), {
    field: "key",
    message: keyRefused,
  });
  assert.deepEqual(connectProblem(problem(400, "invalid_request")), {
    field: "limit",
    message: "Check the limit, then try Connect Boat again.",
  });
  assert.deepEqual(connectProblem(problem(409, "conflict")), {
    field: null,
    copy: {
      title: "Unable to connect Boat",
      body: "Reload the page, then try Connect Boat again.",
    },
  });
  assert.deepEqual(connectProblem(problem(503, "unavailable")), {
    field: null,
    copy: { title: "Unable to connect Boat", body: "Try Connect Boat again." },
  });
  assert.deepEqual(connectProblem({ kind: "network" }), {
    field: null,
    copy: {
      title: "Unable to connect Boat",
      body: "Check the connection, then try Connect Boat again.",
    },
  });
  assert.deepEqual(limitSaveProblem(problem(503, "unavailable")), {
    field: null,
    copy: { title: "Unable to save the limit", body: "Try Save again." },
  });
});

test("every Boat account has Disconnect, and only an accepted key has Change key", () => {
  assert.deepEqual(boatPanelModel(notConnected()), { kind: "none" });
  assert.deepEqual(boatPanelModel(overflowSettings()), {
    kind: "accepted",
    name: "northstar",
    actions: { disconnect: true, changeKey: true },
  });
  assert.deepEqual(boatPanelModel(rejected("2026-11-12T20:00:00Z")), {
    kind: "rejected",
    name: "northstar",
    actions: { disconnect: true, changeKey: false },
  });
});
