import assert from "node:assert/strict";
import test from "node:test";

import type { ApiFailure } from "../src/api/client.ts";
import type {
  PaymentOutcome,
  PlanUsagePage,
} from "../src/api/billing-types.ts";
import type { PaymentSubmission } from "../src/api/billing-payment.ts";
import { usageStore } from "../src/state/billing.ts";
import { createPagePayment } from "../src/state/billing-flows.ts";
import type { PageSnapshot } from "../src/state/billing-page.ts";
import type { PaymentSnapshot } from "../src/state/billing-payment.ts";
import { outcomeModel } from "../src/view/billing-outcome.ts";
import { moveToFreeFailure } from "../src/view/billing-uncertain.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import {
  conflict,
  offer,
  selection,
  signedInWith,
  teamOffer,
  unavailable,
  usagePage,
} from "./billing-support.ts";
import { deferred, response, settled } from "./support.ts";

const workspace = { id: workspaceId, name: "mokly-ai" };
const oneBlock = offer(selection("team", 1), "5900", "35000", "3");
const addOns: PaymentSubmission = { kind: "add-ons", quote: oneBlock };
const noClaim = /stays as it was|did not start|charged|took no payment/;

/** Plan and usage on Team with `blocks` saved blocks of extra starts. */
function teamUsage(blocks = 0): PlanUsagePage {
  const page = usagePage({
    current: teamOffer,
    offer: oneBlock,
    crossing_day: null,
  });
  page.status.selection = selection("team", blocks);
  return page;
}

function ready(page: PlanUsagePage): PageSnapshot<PlanUsagePage> {
  return { status: "ready", page, refreshFailure: null };
}

function failed(failure: ApiFailure, reading = false): PaymentSnapshot {
  return { status: "failed", submission: addOns, failure, reading };
}

const lost: ApiFailure = { kind: "network" };
const serverError: ApiFailure = {
  kind: "problem",
  status: 503,
  problem: unavailable,
};
const unreadable: ApiFailure = { kind: "malformed", status: 200 };
const plainConflict: ApiFailure = {
  kind: "problem",
  status: 409,
  problem: conflict,
};

test("a lost answer after an applied choice says only what the next Plan and usage read proves", async () => {
  const held = deferred<Response>();
  const { app, calls } = await signedInWith(
    "admin",
    response(teamUsage()),
    new Error("connection lost"),
    held.promise,
  );
  const usage = usageStore(app, workspaceId);
  await usage.load();
  const payment = createPagePayment(app, workspaceId);
  const sending = payment.submit(addOns);
  await settled();
  const reading = outcomeModel(
    payment.state.get(),
    workspace,
    usage.state.get(),
  );
  assert.equal(reading?.title, "Unable to confirm this payment");
  assert.doesNotMatch(reading?.body ?? "", noClaim);
  held.resolve(response(teamUsage(1)));
  await sending;
  const after = outcomeModel(payment.state.get(), workspace, usage.state.get());
  assert.deepEqual(
    [after?.title, after?.body, after?.actions.map((item) => item.kind)],
    [
      "Team with 1 block of starts is your current plan",
      "Plan and usage shows it.",
      ["link"],
    ],
  );
  assert.deepEqual(
    calls.map((call) => `${call.init.method} ${call.url}`),
    [
      `GET /api/workspaces/${workspaceId}/billing`,
      `PUT /api/workspaces/${workspaceId}/billing/add-ons`,
      `GET /api/workspaces/${workspaceId}/billing`,
    ],
  );
});

test("Try again after an applied choice shows the choice as current, not a payment in progress", async () => {
  const { app } = await signedInWith(
    "admin",
    response(teamUsage()),
    new Error("connection lost"),
    // The read after the lost answer came before the change took effect.
    response(teamUsage()),
    response(conflict, 409),
    response(teamUsage(1)),
  );
  const usage = usageStore(app, workspaceId);
  await usage.load();
  const payment = createPagePayment(app, workspaceId);
  await payment.submit(addOns);
  const unknown = outcomeModel(
    payment.state.get(),
    workspace,
    usage.state.get(),
  );
  assert.deepEqual(
    [unknown?.title, unknown?.actions.map((item) => item.kind)],
    ["Unable to confirm this payment", ["link", "retry"]],
  );
  // Only a click repeats the payment.
  await payment.retry();
  const again = outcomeModel(payment.state.get(), workspace, usage.state.get());
  assert.equal(
    again?.title,
    "Team with 1 block of starts is your current plan",
  );
});

test("a failure that proves no result follows the fresh read, and a refusal keeps its words", () => {
  const waits = teamUsage();
  waits.status.payment_pending = true;
  const refreshFailed: PageSnapshot<PlanUsagePage> = {
    status: "ready",
    page: teamUsage(1),
    refreshFailure: serverError,
  };
  const cases: [
    string,
    PaymentSnapshot,
    PageSnapshot<PlanUsagePage>,
    string,
    string[],
  ][] = [
    [
      "reading",
      failed(lost, true),
      ready(teamUsage(1)),
      "Unable to confirm this payment",
      ["link", "retry"],
    ],
    [
      "no answer",
      failed(lost),
      ready(teamUsage()),
      "Unable to confirm this payment",
      ["link", "retry"],
    ],
    [
      "server error",
      failed(serverError),
      ready(teamUsage()),
      "Unable to confirm this payment",
      ["link", "retry"],
    ],
    [
      "unreadable success",
      failed(unreadable),
      ready(teamUsage()),
      "Unable to confirm this payment",
      ["link", "retry"],
    ],
    [
      "plain conflict",
      failed(plainConflict),
      ready(teamUsage()),
      "Unable to confirm this payment",
      ["link", "retry"],
    ],
    [
      "failed read",
      failed(lost),
      refreshFailed,
      "Unable to confirm this payment",
      ["link", "reload"],
    ],
    [
      "current",
      failed(serverError),
      ready(teamUsage(1)),
      "Team with 1 block of starts is your current plan",
      ["link"],
    ],
    [
      "waits",
      failed(plainConflict),
      ready(waits),
      "A payment or a billing change is in progress",
      ["link"],
    ],
    [
      "403",
      failed({
        kind: "problem",
        status: 403,
        problem: { code: "access_denied", message: "" },
      }),
      ready(teamUsage()),
      "Only an Admin or a Billing person can pay",
      [],
    ],
    [
      "400",
      failed({
        kind: "problem",
        status: 400,
        problem: { code: "invalid_request", message: "" },
      }),
      ready(teamUsage()),
      "The payment did not start",
      ["retry"],
    ],
  ];
  for (const [name, snapshot, usage, title, actions] of cases) {
    const model = outcomeModel(snapshot, workspace, usage);
    assert.deepEqual(
      [model?.title, model?.actions.map((item) => item.kind)],
      [title, actions],
      name,
    );
    if (title === "Unable to confirm this payment") {
      assert.doesNotMatch(model?.body ?? "", noClaim, name);
    }
  }
});

test("a payment that waits on its Stripe page names that page, not a bank", () => {
  const outcome = {
    state: "payment_required",
    payment_url: "https://invoice.stripe.test/i/in_test",
  } satisfies PaymentOutcome;
  const model = outcomeModel(
    { status: "settled", submission: addOns, outcome, reading: false },
    workspace,
    ready(teamUsage()),
  );
  assert.deepEqual(
    [model?.title, model?.body, model?.actions.map((item) => item.label)],
    [
      "Your payment is not finished",
      "Finish it on its Stripe page. Your plan changes when Stripe confirms the payment.",
      ["Open the Stripe page", "Check again"],
    ],
  );
});

test("Move to Free with no result says only where the plan shows", () => {
  assert.deepEqual(moveToFreeFailure(lost, "Team", "3\u00a0December"), {
    tone: "warning",
    title: "Unable to confirm Move to Free",
    body: "ciproxy did not get the result. Plan and usage shows whether Team ends on 3\u00a0December.",
    retryLabel: "Try again",
    // Move to Free may have taken effect, so the dialog does not offer Keep.
    dismissLabel: "Close",
  });
  for (const failure of [serverError, unreadable, plainConflict]) {
    assert.equal(
      moveToFreeFailure(failure, "Team", "3\u00a0December").title,
      "Unable to confirm Move to Free",
    );
  }
  const refused: ApiFailure = {
    kind: "problem",
    status: 400,
    problem: { code: "invalid_request", message: "" },
  };
  assert.deepEqual(moveToFreeFailure(refused, "Team", "3\u00a0December"), {
    tone: "danger",
    title: "Move to Free did not finish",
    body: "Your plan stays as it was. Try again.",
    retryLabel: "Move to Free",
    dismissLabel: "Keep Team",
  });
});
