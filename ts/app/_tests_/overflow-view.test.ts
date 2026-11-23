import assert from "node:assert/strict";
import test from "node:test";

import type { ApiFailure } from "../src/api/client.ts";
import type { ProblemCode, ProviderSettings } from "../src/api/schema.ts";
import {
  connectedSummary,
  overflowNotices,
  panelProviders,
  providerPanelModel,
  providerProgress,
  spendFigures,
  waitOptions,
} from "../src/view/overflow.ts";
import {
  checkProblem,
  connectProblem,
  limitSaveProblem,
} from "../src/view/overflow-failures.ts";
import { providerCopy } from "../src/view/providers.ts";
import {
  boatSpend,
  fixtureProvider,
  notConnected,
  overflowSettings,
} from "../scripts/fixtures-boat.ts";

// A zone west of UTC shows a date that a local-time format would move.
process.env.TZ = "America/Los_Angeles";

const keyRefused = providerCopy.boat.keyError;

const rejectedAccount = (at: string) => ({
  connected_at: "2026-11-02T09:00:00Z",
  name: "northstar",
  rejected_at: at,
});

const rejected = (at: string) =>
  overflowSettings({ account: rejectedAccount(at) });

function only(settings: { providers: ProviderSettings[] }): ProviderSettings {
  const [provider] = settings.providers;
  assert.notEqual(provider, undefined);
  return provider ?? fixtureProvider();
}

function problem(status: number, code: ProblemCode, named = true): ApiFailure {
  return {
    kind: "problem",
    status,
    problem: {
      code,
      message: "API copy",
      ...(code === "provider_key_rejected" && named
        ? { provider: "boat" }
        : {}),
    },
  };
}

test("the spend of a provider shows confirmed and unconfirmed parts, and only the API flag reaches the limit", () => {
  assert.deepEqual(spendFigures(fixtureProvider()), {
    name: "Boat",
    confirmed: "$48.10",
    unconfirmed: "$13.10",
    settled: "$1.20",
    limit: "$200",
    confirmedShare: 0.2405,
    unconfirmedShare: 0.0655,
    reached: false,
  });
  const rounded = fixtureProvider({
    spend: boatSpend({ confirmed_cents: "15000", unconfirmed_cents: "5000" }),
  });
  assert.equal(spendFigures(rounded)?.reached, false);
  const flagged = fixtureProvider({
    spend: boatSpend({ limit_reached: true }),
  });
  assert.equal(spendFigures(flagged)?.reached, true);
  const unknown = fixtureProvider({
    spend: boatSpend({ bounded: false, limit_reached: null }),
  });
  assert.equal(spendFigures(unknown)?.reached, false);
  assert.equal(
    spendFigures(
      fixtureProvider({ spend: boatSpend({ settled_bound_cents: "0" }) }),
    )?.settled,
    null,
  );
  assert.equal(spendFigures(fixtureProvider({ spend: null })), null);
});

test("each provider notice names its provider and when jobs start there again", () => {
  assert.deepEqual(overflowNotices(overflowSettings()), []);
  assert.deepEqual(overflowNotices(rejected("2026-11-12T20:00:00Z")), [
    {
      tone: "danger",
      title: "Boat rejected the API key on 12\u00a0November",
      body: "ciproxy starts no new jobs on Boat until you connect a new key.",
    },
  ]);
  const reached = overflowSettings({
    spend: boatSpend({ limit_reached: true }),
  });
  assert.deepEqual(overflowNotices(reached), [
    {
      tone: "warning",
      title: "Boat spend reached the $200 limit for November",
      body: "ciproxy starts no new jobs on Boat until 1\u00a0December, or until you raise the limit.",
    },
  ]);
  assert.deepEqual(
    overflowNotices({ ...reached, next_month_at: "2027-01-01T00:00:00Z" }),
    [
      {
        tone: "warning",
        title: "Boat spend reached the $200 limit for December",
        body: "ciproxy starts no new jobs on Boat until 1\u00a0January, or until you raise the limit.",
      },
    ],
  );
  const both = overflowSettings({
    account: rejectedAccount("2026-11-12T20:00:00Z"),
    spend: boatSpend({ limit_reached: true }),
  });
  assert.deepEqual(
    overflowNotices(both).map((notice) => notice.tone),
    ["danger"],
  );
  assert.deepEqual(
    overflowNotices(
      overflowSettings({
        spend: boatSpend({ bounded: false, limit_reached: null }),
      }),
    ),
    [
      {
        tone: "warning",
        title: "Boat costs are not known yet",
        body: "ciproxy starts no new jobs on Boat until Boat reports the cost of each start.",
      },
    ],
  );
  assert.deepEqual(overflowNotices(overflowSettings({ available: false })), [
    {
      tone: "warning",
      title: "Boat is unavailable right now",
      body: "ciproxy starts no new jobs on Boat until it is available again. Jobs skip Boat and go on to the next place of their label.",
    },
  ]);
  assert.deepEqual(overflowNotices(notConnected(false)), []);
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

test("Setup counts a provider as connected only with an accepted key, and a skip only for its person", () => {
  const ready = { status: "ready" as const, refreshFailure: null };
  assert.equal(
    connectedSummary(overflowSettings()),
    "Connected to the Boat account northstar · spend limit $200 a month",
  );
  assert.equal(connectedSummary(rejected("2026-11-12T20:00:00Z")), null);
  assert.deepEqual(
    providerProgress(
      { ...ready, settings: overflowSettings() },
      "mira",
      "mira",
    ),
    {
      connected:
        "Connected to the Boat account northstar · spend limit $200 a month",
      skipped: false,
    },
  );
  const none = { ...ready, settings: overflowSettings({ account: null }) };
  assert.deepEqual(providerProgress(none, "mira", "mira"), {
    connected: null,
    skipped: true,
  });
  assert.deepEqual(providerProgress(none, "mira", "sasha"), {
    connected: null,
    skipped: false,
  });
  assert.deepEqual(providerProgress({ status: "loading" }, null, "mira"), {
    connected: null,
    skipped: false,
  });
});

test("a key that its provider refused belongs to the key field, a refused limit to the limit field", () => {
  assert.deepEqual(
    connectProblem(problem(422, "provider_key_rejected"), "boat"),
    { field: "key", message: keyRefused },
  );
  assert.deepEqual(
    checkProblem(problem(422, "provider_key_rejected"), "boat"),
    { field: "key", message: keyRefused },
  );
  assert.deepEqual(
    checkProblem(problem(422, "provider_key_rejected", false), "boat"),
    {
      field: null,
      copy: { title: "Unable to check the key", body: "Try Check key again." },
    },
  );
  assert.deepEqual(checkProblem(problem(503, "unavailable"), "boat"), {
    field: null,
    copy: { title: "Unable to check the key", body: "Try Check key again." },
  });
  assert.deepEqual(checkProblem({ kind: "network" }, "boat"), {
    field: null,
    copy: {
      title: "Unable to check the key",
      body: "Check the connection, then try Check key again.",
    },
  });
  assert.deepEqual(connectProblem(problem(400, "invalid_request"), "boat"), {
    field: "limit",
    message: "Check the limit, then try Connect Boat again.",
  });
  assert.deepEqual(connectProblem(problem(409, "conflict"), "boat"), {
    field: null,
    copy: {
      title: "Unable to connect Boat",
      body: "Reload the page, then try Connect Boat again.",
    },
  });
  assert.deepEqual(connectProblem(problem(503, "unavailable"), "boat"), {
    field: null,
    copy: { title: "Unable to connect Boat", body: "Try Connect Boat again." },
  });
  assert.deepEqual(connectProblem({ kind: "network" }, "boat"), {
    field: null,
    copy: {
      title: "Unable to connect Boat",
      body: "Check the connection, then try Connect Boat again.",
    },
  });
  assert.deepEqual(limitSaveProblem(problem(503, "unavailable"), "boat"), {
    field: null,
    copy: { title: "Unable to save the limit", body: "Try Save limit again." },
  });
});

test("every account has Disconnect, and only an accepted key of an available provider has Change key", () => {
  assert.deepEqual(providerPanelModel(only(notConnected())), { kind: "none" });
  assert.deepEqual(providerPanelModel(fixtureProvider()), {
    kind: "accepted",
    name: "northstar",
    actions: { disconnect: true, changeKey: true },
  });
  assert.deepEqual(providerPanelModel(only(rejected("2026-11-12T20:00:00Z"))), {
    kind: "rejected",
    name: "northstar",
    actions: { disconnect: true, changeKey: false },
  });
  assert.deepEqual(providerPanelModel(fixtureProvider({ available: false })), {
    kind: "accepted",
    name: "northstar",
    actions: { disconnect: true, changeKey: false },
  });
});

test("Settings shows a panel for each provider that is connected or available", () => {
  const panels = (settings: { providers: ProviderSettings[] }) =>
    panelProviders({ ...overflowSettings(), ...settings }).map(
      (provider) => provider.provider,
    );
  assert.deepEqual(panels(overflowSettings()), ["boat"]);
  assert.deepEqual(panels(overflowSettings({ available: false })), ["boat"]);
  assert.deepEqual(panels(notConnected()), ["boat"]);
  assert.deepEqual(panels(notConnected(false)), []);
});
