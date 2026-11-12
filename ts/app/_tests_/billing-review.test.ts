import assert from "node:assert/strict";
import test from "node:test";

import type { ApiResult } from "../src/api/client.ts";
import type {
  CheckoutReviewPage,
  PaymentOutcome,
  PlanSelection,
} from "../src/api/billing-types.ts";
import { planSubmission, submitPayment } from "../src/api/billing-payment.ts";
import { createReview } from "../src/state/billing-flows.ts";
import {
  createReviewStore,
  reviewedChoice,
} from "../src/state/billing-review.ts";
import { paymentGate, usageStore } from "../src/state/billing.ts";
import { reviewModel } from "../src/view/billing-checkout.ts";
import { checkoutSummary } from "../src/view/billing-checkout-summary.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import {
  offer,
  paidQuote,
  paidReview,
  review,
  selection,
  signedInWith,
  teamOffer,
  unavailable,
  usagePage,
} from "./billing-support.ts";
import { appWith, deferred, response, settled } from "./support.ts";

const oneBlock = offer(selection("team", 1), "5900", "35000", "3");

test("the review shows only a price of exactly the wanted choice", async () => {
  const reads: PlanSelection[] = [];
  const replies: ReturnType<typeof deferred<ApiResult<CheckoutReviewPage>>>[] =
    [];
  const store = createReviewStore({
    initial: selection("team"),
    read: (wanted) => {
      reads.push(wanted);
      const reply = deferred<ApiResult<CheckoutReviewPage>>();
      replies.push(reply);
      return reply.promise;
    },
    report: (result) => result,
  });
  void store.ensure();
  void store.edit({ start_blocks: 1, machines: 0 });
  assert.equal(reviewedChoice(store.state.get()), null);
  replies[1]?.resolve({ ok: true, value: review(oneBlock) });
  await settled();
  // The older review of the first choice arrives late and is dropped.
  replies[0]?.resolve({ ok: true, value: review(teamOffer) });
  await settled();
  assert.equal(reviewedChoice(store.state.get())?.choice, oneBlock);
  void store.edit({ start_blocks: 2, machines: 0 });
  assert.equal(reviewedChoice(store.state.get()), null);
  replies[2]?.resolve({ ok: true, value: review(oneBlock) });
  await settled();
  // A review of another choice than the wanted one is never shown.
  assert.equal(reviewedChoice(store.state.get()), null);
  assert.deepEqual(reads, [
    selection("team"),
    selection("team", 1),
    selection("team", 2),
  ]);
});

test("a review sends every query value and a failed review shows no price", async () => {
  const { app, calls } = appWith(
    response(unavailable, 503),
    response(review(teamOffer)),
  );
  const store = createReview(app, workspaceId, selection("team"));
  await store.ensure();
  assert.equal(
    calls[0]?.url,
    `/api/workspaces/${workspaceId}/checkout/review?tier=team&start_blocks=0&machines=0`,
  );
  assert.equal(store.state.get().review.status, "failed");
  assert.equal(reviewedChoice(store.state.get()), null);
  await store.retry();
  assert.deepEqual(reviewedChoice(store.state.get())?.choice, teamOffer);
});

test("each submission sends the reviewed or quoted choice to its route", async () => {
  const confirmed = { state: "confirmed" } satisfies PaymentOutcome;
  const { app, calls } = appWith(
    response(confirmed),
    response(confirmed),
    response(confirmed),
  );
  const prorated = paidReview(oneBlock);
  await submitPayment(app.api, workspaceId, {
    kind: "plan",
    review: review(oneBlock),
  });
  await submitPayment(app.api, workspaceId, { kind: "plan", review: prorated });
  await submitPayment(app.api, workspaceId, {
    kind: "add-ons",
    quote: oneBlock,
  });
  const sent = calls.map((call) => [
    call.init.method,
    call.url.replace(`/api/workspaces/${workspaceId}`, ""),
    JSON.parse(String(call.init.body)),
  ]);
  assert.deepEqual(sent, [
    ["POST", "/checkout", selection("team", 1)],
    ["POST", "/billing/change", { quote_id: paidQuote.id }],
    ["PUT", "/billing/add-ons", { start_blocks: 1, machines: 0 }],
  ]);
});

test("a review keeps its stored paid quote and normalizes an absent quote", async () => {
  const unpriced = {
    kind: "blocked",
    title: "Unable to price this change",
    body: "The price of this choice did not load. Try again.",
    action: { kind: "retry-review", label: "Try again" },
  };
  const quotes = [
    { quote: paidQuote, expected: { kind: "ready" } },
    { quote: null, expected: unpriced },
    { quote: undefined, expected: unpriced },
  ];
  for (const { quote, expected } of quotes) {
    const { app } = await signedInWith(
      "admin",
      response(usagePage()),
      response({ ...paidReview(), quote }),
    );
    const usage = usageStore(app, workspaceId);
    await usage.load();
    const store = createReview(app, workspaceId, selection("team"));
    await store.ensure();
    assert.equal(store.state.get().review.status, "ready");
    assert.deepEqual(reviewedChoice(store.state.get())?.quote, quote ?? null);
    const current = usage.state.get();
    const summary = checkoutSummary({
      review: store.state.get(),
      gate: paymentGate(app, workspaceId),
      usage: current.status === "ready" ? current.page : null,
      payment: { status: "idle" },
      workspace: "mokly-ai",
    });
    assert.deepEqual(
      summary.kind === "ready" ? { kind: summary.kind } : summary,
      expected,
    );
  }
  const { app } = appWith(response({ ...review(), quote: undefined }));
  const store = createReview(app, workspaceId, selection("team"));
  await store.ensure();
  assert.equal(reviewedChoice(store.state.get())?.quote, null);
});

test("a paid review without its quote cannot build a payment", () => {
  for (const quote of [null, undefined]) {
    assert.equal(planSubmission({ ...paidReview(), quote }), null);
  }
});

test("a paid review shows today's charge apart from the monthly total, with the day of later charges", () => {
  // Business with 2 blocks of starts and 1 extra own machine. The API gives
  // a prorated charge for today that differs from the monthly total, and a
  // period that ends at midnight on the billing day.
  const paid = paidReview(
    offer(selection("business", 2, 1), "33900", "170000", "11"),
    {
      today_cents: "21037",
      billing_day: 17,
      period_end: "2026-12-17T00:00:00Z",
    },
  );
  const model = reviewModel(paid, "mokly-ai");
  assert.deepEqual(
    model.rows.map((row) => [row.label, row.value]),
    [
      ["Plan", "Business"],
      ["Extra starts", "2 blocks of starts"],
      ["Extra own machines", "1 extra own machine"],
      ["Total a month", "$339\u00a0a\u00a0month"],
      ["Charged today", "$210.37"],
    ],
  );
  assert.match(
    model.note,
    /card on file \$210\.37 for the rest of this period, which ends on 16\u00a0December, then \$339 on the 17th of each month\./,
  );
});
