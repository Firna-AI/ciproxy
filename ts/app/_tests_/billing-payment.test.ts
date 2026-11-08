import assert from "node:assert/strict";
import test from "node:test";

import type { ApiResult } from "../src/api/client.ts";
import type { PaymentOutcome } from "../src/api/billing-types.ts";
import type { PaymentSubmission } from "../src/api/billing-payment.ts";
import { createPaymentStore } from "../src/state/billing-payment.ts";
import { usageStore } from "../src/state/billing.ts";
import { createPagePayment } from "../src/state/billing-flows.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import {
  offer,
  review,
  selection,
  signedInWith,
  unavailable,
  usagePage,
} from "./billing-support.ts";
import { deferred, response, settled } from "./support.ts";

const team: PaymentSubmission = { kind: "plan", review: review() };
const business: PaymentSubmission = {
  kind: "plan",
  review: review(offer(selection("business"), "29900", "150000", "10")),
};

function paymentWith(outcome: PaymentOutcome, held: Promise<void>) {
  const sent: PaymentSubmission[] = [];
  const store = createPaymentStore({
    send: async (submission): Promise<ApiResult<PaymentOutcome>> => {
      sent.push(submission);
      return { ok: true, value: outcome };
    },
    mutate: (run) => run(),
    // The page still holds its capabilities from before the payment.
    canStart: () => true,
    leave: () => undefined,
    settled: () => held,
  });
  return { store, sent };
}

test("one payment runs until the server state after it is read again", async () => {
  const held = deferred<void>();
  const { store, sent } = paymentWith({ state: "confirming" }, held.promise);
  const first = store.submit(team);
  await settled();
  // While the read after the outcome is held, nothing sends again.
  void store.submit(business);
  void store.submit(team);
  void store.retry();
  await settled();
  assert.deepEqual(sent, [team]);
  held.resolve();
  await first;
  // After that read, Retry sends the same submission once more.
  const again = store.retry();
  void store.retry();
  held.resolve();
  await again;
  assert.deepEqual(sent, [team, team]);
});

test("a failed read after a payment keeps new payments closed", async () => {
  const { app, calls } = await signedInWith(
    "admin",
    response(usagePage()),
    response({ state: "confirmed" } satisfies PaymentOutcome),
    response(unavailable, 503),
  );
  await usageStore(app, workspaceId).load();
  const payment = createPagePayment(app, workspaceId);
  await payment.submit(team);
  assert.equal(calls.length, 3);
  assert.equal(calls[1]?.url, `/api/workspaces/${workspaceId}/checkout`);
  const usage = usageStore(app, workspaceId).state.get();
  assert.equal(
    usage.status === "ready" && usage.refreshFailure?.kind,
    "problem",
  );
  await payment.submit(business);
  assert.equal(calls.length, 3);
});
