import assert from "node:assert/strict";
import test from "node:test";

import type {
  PlanCard,
  PlanOffer,
  PlansPage,
} from "../src/api/billing-types.ts";
import type { PaymentSnapshot } from "../src/state/billing-payment.ts";
import { addOnQuote, suggestedAddOns } from "../src/view/billing-addons.ts";
import { outcomeModel } from "../src/view/billing-outcome.ts";
import { plansModel } from "../src/view/billing-plans.ts";
import { usageModel } from "../src/view/billing-usage.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import {
  freeOffer,
  offer,
  review,
  selection,
  teamOffer,
  usagePage,
} from "./billing-support.ts";

const workspace = { id: workspaceId, name: "mokly-ai" };
const businessOffer = offer(selection("business"), "29900", "150000", "10");

function card(plan: PlanOffer, overrides: Partial<PlanCard> = {}): PlanCard {
  return {
    plan,
    current: false,
    covers_use: false,
    can_choose: true,
    history_days: 30,
    organizations: null,
    features: [],
    initial_add_ons: { start_blocks: 0, machines: 0 },
    ...overrides,
  };
}

function plansPage(): PlansPage {
  return {
    usage: usagePage(),
    plans: [
      card(freeOffer, { current: true, can_choose: false }),
      card(teamOffer, {
        covers_use: true,
        initial_add_ons: { start_blocks: 1, machines: 0 },
      }),
      card(businessOffer),
    ],
  };
}

test("a payer chooses each other plan, and every other role reads who can", () => {
  const payer = plansModel(plansPage(), workspaceId).cards;
  assert.deepEqual(
    payer.map((item) => item.action.kind),
    ["current", "choose", "choose"],
  );
  const team = payer[1]?.action;
  assert.equal(team?.kind === "choose" && team.primary, true);
  assert.equal(
    team?.kind === "choose" && team.href,
    `/checkout?workspace=${workspaceId}&tier=team&start_blocks=1&machines=0`,
  );
  const page = plansPage();
  page.usage.status.can_pay = false;
  assert.deepEqual(
    plansModel(page, workspaceId).cards.map((item) => item.action.kind),
    ["current", "ask", "ask"],
  );
});

test("while a payment is pending, no plan opens a second payment", () => {
  const page = plansPage();
  page.usage.status.payment_pending = true;
  for (const item of page.plans) item.can_choose = false;
  const model = plansModel(page, workspaceId);
  assert.equal(model.waiting, true);
  for (const item of model.cards) {
    assert.equal("enabled" in item.action && item.action.enabled, false);
  }
});

test("a plan card lists its job history before the features that come later", () => {
  const page = plansPage();
  page.plans[1] = card(teamOffer, {
    covers_use: true,
    features: [
      { name: "Sticky Docker disks", coming_later: false },
      { name: "Alerts by email and Slack", coming_later: true },
    ],
  });
  assert.deepEqual(plansModel(page, workspaceId).cards[1]?.features, [
    { name: "Sticky Docker disks", comingLater: false },
    { name: "30 days of job history", comingLater: false },
    { name: "Alerts by email and Slack", comingLater: true },
  ]);
});

test("on a paid plan, Free asks before Move to Free", () => {
  const page = plansPage();
  page.usage.status.selection = selection("team");
  page.plans[0] = card(freeOffer);
  page.plans[1] = card(teamOffer, { current: true, can_choose: false });
  assert.equal(
    plansModel(page, workspaceId).cards[0]?.action.kind,
    "move-to-free",
  );
});

test("add-ons save only a choice that the API priced", () => {
  const page = usagePage({
    current: teamOffer,
    offer: offer(selection("team", 1), "5900", "35000", "3"),
  });
  page.status.selection = selection("team");
  assert.equal(
    addOnQuote(page, { start_blocks: 0, machines: 0 }).kind,
    "saved",
  );
  assert.equal(
    addOnQuote(page, { start_blocks: 1, machines: 0 }).kind,
    "offer",
  );
  assert.equal(
    addOnQuote(page, { start_blocks: 2, machines: 0 }).kind,
    "unknown",
  );
  assert.deepEqual(suggestedAddOns(page), { start_blocks: 1, machines: 0 });
  assert.equal(suggestedAddOns(usagePage()), null);
});

test("a payment outcome offers the way to recover that the server allows", () => {
  const submission = { kind: "plan", review: review() } as const;
  // A fresh Plan and usage read that shows Free and nothing that waits.
  const current = {
    status: "ready",
    page: usagePage(),
    refreshFailure: null,
  } as const;
  const settled = (outcome: PaymentSnapshot & { status: "settled" }) =>
    outcomeModel(outcome, workspace, current)?.actions.map(
      (action) => action.kind,
    );
  const bank = settled({
    status: "settled",
    submission,
    reading: false,
    outcome: { state: "payment_required", payment_url: "https://pay.test/i" },
  });
  assert.deepEqual(bank, ["link", "retry"]);
  const cardFailed = settled({
    status: "settled",
    submission,
    reading: false,
    outcome: { state: "payment_required", payment_url: null },
  });
  assert.deepEqual(cardFailed, ["portal", "retry"]);
  const failed = (status: number) =>
    outcomeModel(
      {
        status: "failed",
        submission,
        reading: false,
        failure: {
          kind: "problem",
          status,
          problem: { code: "conflict", message: "" },
        },
      },
      workspace,
      current,
    )?.actions.map((action) => action.kind);
  // A plain 409 and a 503 prove no result, and the fresh read shows neither
  // the choice nor a wait: the result stays unknown, with Try again.
  assert.deepEqual(failed(409), ["link", "retry"]);
  assert.deepEqual(failed(403), []);
  assert.deepEqual(failed(503), ["link", "retry"]);
});

test("Plan and usage forecasts only before the limit and names the end of a plan", () => {
  const free = usageModel(usagePage(), "mokly-ai");
  assert.deepEqual(
    free.notices.map((notice) => notice.kind === "pace" && notice.action),
    ["compare"],
  );
  const over = usagePage({
    starts: "2214",
    starts_over: "214",
    crossing_day: null,
  });
  const passed = usageModel(over, "mokly-ai");
  assert.deepEqual(passed.notices, []);
  assert.match(passed.starts.detail ?? "", /^214 over/);
  const paid = usagePage({ current: teamOffer, crossing_day: null });
  paid.status.selection = selection("team");
  paid.status.cancel_at = "2026-12-01T00:00:00Z";
  paid.status.payment_pending = true;
  const ending = usageModel(paid, "mokly-ai");
  assert.deepEqual(
    ending.notices.map((notice) => notice.kind),
    ["confirming", "ends"],
  );
  assert.match(
    ending.notices[1]?.title ?? "",
    /^Team ends on 30\u00a0November$/,
  );
  // A period ends at its exclusive end: a noon end includes its own date, and
  // a midnight end does not.
  const period = (start: string, end: string) => {
    const page = usagePage();
    page.status.period_start = start;
    page.status.period_end = end;
    return usageModel(page, "mokly-ai").period;
  };
  assert.equal(
    period("2026-11-06T12:00:00Z", "2026-12-06T12:00:00Z"),
    "6\u00a0November\u00a0– 6\u00a0December",
  );
  assert.equal(
    period("2026-11-04T00:00:00Z", "2026-12-04T00:00:00Z"),
    "4\u00a0November\u00a0– 3\u00a0December",
  );
  // An active paid plan below its starts and over its machines: each value
  // shows its own field, with the renewal and the crossing dates.
  const active = usagePage({
    current: offer(selection("team", 1, 1), "7900", "35000", "4"),
    offer: offer(selection("team", 2, 2), "10900", "45000", "5"),
    starts: "8640",
    machines: "5",
    machines_over: "1",
    expected_starts: "38750",
    crossing_day: "2026-11-27",
  });
  active.status.selection = selection("team", 1, 1);
  active.status.subscription = "active";
  active.status.period_start = "2026-11-04T10:30:00Z";
  active.status.period_end = "2026-12-04T10:30:00Z";
  const shown = usageModel(active, "mokly-ai");
  assert.deepEqual(
    [
      shown.plan.price,
      shown.starts.value,
      shown.starts.detail,
      shown.machines.value,
      shown.machines.detail,
    ],
    [
      "$79\u00a0a\u00a0month\u00a0· renews on 4\u00a0December",
      "8,640 of 35,000",
      "About 38,750 by the end of the period at this pace.",
      "5 of 4",
      "1 over. Every machine runs jobs.",
    ],
  );
  assert.equal(
    shown.notices.find((notice) => notice.kind === "pace")?.title,
    "At this pace, mokly-ai passes the 35,000 starts of Team on 27\u00a0November",
  );
});
