import assert from "node:assert/strict";
import test from "node:test";

import { planSubmission } from "../src/api/billing-payment.ts";
import type { PlanUsagePage } from "../src/api/billing-types.ts";
import {
  checkoutReviewAgain,
  createPagePayment,
  createReview,
} from "../src/state/billing-flows.ts";
import { reviewedChoice } from "../src/state/billing-review.ts";
import { paymentGate, usageStore } from "../src/state/billing.ts";
import { checkoutSummary } from "../src/view/billing-checkout-summary.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import {
  billingApiPaths,
  offer,
  paidQuote,
  paidReview,
  selection,
  signedInWith,
  teamOffer,
  usagePage,
} from "./billing-support.ts";
import { deferred, response } from "./support.ts";

const business = offer(selection("business"), "29900", "150000", "10");
const quoted = paidReview(business, { today_cents: "13333" });
const nextQuote = {
  id: "123e4567-e89b-12d3-a456-426614174001",
  expires_at: "2026-11-06T12:31:00Z",
};

function currentTeam(): PlanUsagePage {
  const page = usagePage({ current: teamOffer, crossing_day: null });
  page.status.selection = selection("team");
  page.status.subscription = "active";
  return page;
}

function currentBusiness(): PlanUsagePage {
  const page = usagePage({ current: business, crossing_day: null });
  page.status.selection = selection("business");
  page.status.subscription = "active";
  return page;
}

test("review again loads before reset and the next click pays the new quote at the same price", async () => {
  const heldReview = deferred<Response>();
  const { app, calls } = await signedInWith(
    "admin",
    response(currentTeam()),
    response(quoted),
    response({ code: "review_required", message: "Review again." }, 409),
    response(currentTeam()),
    heldReview.promise,
    response({ state: "confirmed" }),
    response(currentBusiness()),
  );
  const usage = usageStore(app, workspaceId);
  await usage.load();
  const review = createReview(app, workspaceId, business.selection);
  await review.ensure();
  const shown = reviewedChoice(review.state.get());
  assert.ok(shown !== null);
  const original = planSubmission(shown);
  assert.ok(original !== null);
  const payment = createPagePayment(app, workspaceId);
  await payment.submit(original);
  const transitions: [string, string, boolean][] = [];
  const record = () => {
    transitions.push([
      review.state.get().review.status,
      payment.state.get().status,
      reviewedChoice(review.state.get()) === null,
    ]);
  };
  const stopReview = review.state.subscribe(record);
  const stopPayment = payment.state.subscribe(record);
  const reading = checkoutReviewAgain(review, payment);
  try {
    assert.deepEqual(transitions, [
      ["loading", "failed", true],
      ["loading", "idle", true],
    ]);
    assert.equal(reviewedChoice(review.state.get()), null);
    assert.deepEqual(
      calls.map((call) => call.init.method),
      ["GET", "GET", "POST", "GET", "GET"],
    );
  } finally {
    heldReview.resolve(response({ ...quoted, quote: nextQuote }));
    await reading;
    stopReview();
    stopPayment();
  }
  const current = usage.state.get();
  const summary = checkoutSummary({
    review: review.state.get(),
    payment: payment.state.get(),
    gate: paymentGate(app, workspaceId),
    usage: current.status === "ready" ? current.page : null,
    workspace: "mokly-ai",
  });
  assert.ok(summary.kind === "ready");
  assert.equal(
    summary.model.rows.find((row) => row.label === "Charged today")?.value,
    "$133.33",
  );
  assert.equal(calls.filter((call) => call.init.method === "POST").length, 1);
  await payment.submit(summary.submission);
  assert.deepEqual(
    calls
      .filter((call) => call.init.method === "POST")
      .map((call) => JSON.parse(String(call.init.body))),
    [{ quote_id: paidQuote.id }, { quote_id: nextQuote.id }],
  );
  assert.deepEqual(
    calls.map((call) => `${call.init.method} ${call.url.split("?")[0]}`),
    [
      `GET ${billingApiPaths.usage}`,
      `GET ${billingApiPaths.review}`,
      `POST ${billingApiPaths.change}`,
      `GET ${billingApiPaths.usage}`,
      `GET ${billingApiPaths.review}`,
      `POST ${billingApiPaths.change}`,
      `GET ${billingApiPaths.usage}`,
    ],
  );
});

test("an unfinished paid change retries its original quote without a new review", async () => {
  const pending = currentTeam();
  pending.status.payment_pending = true;
  const { app, calls } = await signedInWith(
    "admin",
    response(currentTeam()),
    response({
      state: "payment_required",
      payment_url: "https://invoice.stripe.test/payment",
    }),
    response(pending),
    response({ state: "confirmed" }),
    response(currentBusiness()),
  );
  await usageStore(app, workspaceId).load();
  const payment = createPagePayment(app, workspaceId);
  await payment.submit({ kind: "plan", review: quoted });
  assert.equal(payment.state.get().status, "settled");
  await payment.retry();
  assert.deepEqual(
    calls
      .filter((call) => call.init.method === "POST")
      .map((call) => JSON.parse(String(call.init.body))),
    [{ quote_id: paidQuote.id }, { quote_id: paidQuote.id }],
  );
  assert.deepEqual(
    calls.map((call) => `${call.init.method} ${call.url}`),
    [
      `GET ${billingApiPaths.usage}`,
      `POST ${billingApiPaths.change}`,
      `GET ${billingApiPaths.usage}`,
      `POST ${billingApiPaths.change}`,
      `GET ${billingApiPaths.usage}`,
    ],
  );
});
