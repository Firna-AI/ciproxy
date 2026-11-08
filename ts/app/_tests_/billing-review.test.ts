import assert from "node:assert/strict";
import test from "node:test";

import type { ApiResult } from "../src/api/client.ts";
import type {
  CheckoutReviewPage,
  PaymentOutcome,
  PlanSelection,
} from "../src/api/billing-types.ts";
import { submitPayment } from "../src/api/billing-payment.ts";
import { createReview } from "../src/state/billing-flows.ts";
import {
  createReviewStore,
  reviewedChoice,
} from "../src/state/billing-review.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import {
  offer,
  review,
  selection,
  teamOffer,
  unavailable,
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
  const prorated = review(oneBlock, { prorated: true });
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
    ["POST", "/billing/change", selection("team", 1)],
    ["PUT", "/billing/add-ons", { start_blocks: 1, machines: 0 }],
  ]);
});
