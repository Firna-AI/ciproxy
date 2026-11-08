import assert from "node:assert/strict";
import test from "node:test";

import type {
  PaymentOutcome,
  PlanUsagePage,
} from "../src/api/billing-types.ts";
import type { AppServices } from "../src/services/services.ts";
import { paymentGate, usageStore } from "../src/state/billing.ts";
import {
  checkoutReload,
  createPagePayment,
  createReview,
} from "../src/state/billing-flows.ts";
import type { ReviewStore } from "../src/state/billing-review.ts";
import {
  checkoutSummary,
  type SummaryState,
} from "../src/view/billing-checkout-summary.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import {
  conflict,
  offer,
  review,
  selection,
  signedInWith,
  unavailable,
  usagePage,
} from "./billing-support.ts";
import { response } from "./support.ts";

/** Team with 2 blocks of starts and 1 extra own machine: nonzero add-ons. */
const reviewed = review(offer(selection("team", 2, 1), "8900", "45000", "4"));
const checkoutUrl = "https://checkout.stripe.test/c/pay/cs_test_gate";

function summaryOf(app: AppServices, store: ReviewStore): SummaryState {
  const usage = usageStore(app, workspaceId).state.get();
  return checkoutSummary({
    review: store.state.get(),
    gate: paymentGate(app, workspaceId),
    usage: usage.status === "ready" ? usage.page : null,
    workspace: "mokly-ai",
  });
}

/** The kind, the title, and the action kind of a summary. */
function shape(summary: SummaryState) {
  return summary.kind === "blocked"
    ? [summary.kind, summary.title, summary.action?.kind ?? null]
    : [summary.kind, summary.kind === "loading" ? summary.text : null, null];
}

function page(change: (value: PlanUsagePage) => void = () => undefined) {
  const value = usagePage();
  change(value);
  return value;
}

test("the payment gate gives one reason, and a role that cannot pay closes it at once", async () => {
  const viewer = await signedInWith("viewer");
  assert.equal(paymentGate(viewer.app, workspaceId), "not-payer");
  const admin = await signedInWith(
    "admin",
    response(usagePage()),
    response(unavailable, 503),
  );
  assert.equal(paymentGate(admin.app, workspaceId), "loading");
  await usageStore(admin.app, workspaceId).load();
  assert.equal(paymentGate(admin.app, workspaceId), "open");
  // A failed refresh keeps the older page, and the state is unknown.
  await usageStore(admin.app, workspaceId).load();
  assert.equal(paymentGate(admin.app, workspaceId), "unknown");
  const cases: [string, PlanUsagePage, string][] = [
    [
      "API says the person cannot pay",
      page((value) => (value.status.can_pay = false)),
      "not-payer",
    ],
    [
      "a payment waits",
      page((value) => (value.status.payment_pending = true)),
      "waiting",
    ],
    [
      "an operation waits",
      page((value) => (value.status.operation_in_progress = true)),
      "waiting",
    ],
    ["nothing waits", page(), "open"],
  ];
  for (const [name, value, reason] of cases) {
    const billing = await signedInWith("billing", response(value));
    await usageStore(billing.app, workspaceId).load();
    assert.equal(paymentGate(billing.app, workspaceId), reason, name);
  }
  const failedFirst = await signedInWith("admin", response(unavailable, 503));
  await usageStore(failedFirst.app, workspaceId).load();
  assert.equal(paymentGate(failedFirst.app, workspaceId), "unknown");
});

test("Checkout names a failed Plan and usage read, keeps the review, and pays only after Try again and a click", async () => {
  const { app, calls, documents } = await signedInWith(
    "admin",
    response(unavailable, 503),
    response(reviewed),
    response(usagePage()),
    response({ state: "open", url: checkoutUrl } satisfies PaymentOutcome),
  );
  const store = createReview(app, workspaceId, reviewed.choice.selection);
  await Promise.all([usageStore(app, workspaceId).load(), store.ensure()]);
  assert.deepEqual(shape(summaryOf(app, store)), [
    "blocked",
    "Unable to load plan and usage",
    "reload",
  ]);
  await checkoutReload(app, workspaceId, store);
  const ready = summaryOf(app, store);
  assert.equal(ready.kind, "ready");
  assert.deepEqual(
    calls.map((call) => call.init.method),
    ["GET", "GET", "GET"],
  );
  // The payment starts only with the click, for the reviewed quantities.
  await createPagePayment(app, workspaceId).submit({
    kind: "plan",
    review: reviewed,
  });
  assert.deepEqual(
    JSON.parse(String(calls[3]?.init.body)),
    selection("team", 2, 1),
  );
  assert.deepEqual(documents, [checkoutUrl]);
});

test("when both Checkout reads fail, one Try again reads both", async () => {
  const { app } = await signedInWith(
    "admin",
    response(unavailable, 503),
    response(unavailable, 503),
    response(usagePage()),
    response(reviewed),
  );
  const store = createReview(app, workspaceId, reviewed.choice.selection);
  await Promise.all([usageStore(app, workspaceId).load(), store.ensure()]);
  assert.deepEqual(shape(summaryOf(app, store)), [
    "blocked",
    "Unable to load plan and usage",
    "reload",
  ]);
  await checkoutReload(app, workspaceId, store);
  assert.equal(summaryOf(app, store).kind, "ready");
});

test("an unknown billing state comes before the words of a refused review", async () => {
  const { app } = await signedInWith(
    "admin",
    response(unavailable, 503),
    response(conflict, 409),
  );
  const store = createReview(app, workspaceId, reviewed.choice.selection);
  await Promise.all([usageStore(app, workspaceId).load(), store.ensure()]);
  assert.deepEqual(shape(summaryOf(app, store)), [
    "blocked",
    "Unable to load plan and usage",
    "reload",
  ]);
});

test("every closed reason of Checkout explains itself", async () => {
  const pending = page((value) => (value.status.payment_pending = true));
  const denied = page((value) => (value.status.can_pay = false));
  const cases: [string, PlanUsagePage | null, unknown[]][] = [
    ["loading", null, ["loading", "Checking your plan", null]],
    [
      "waiting",
      pending,
      ["blocked", "A payment or a billing change is in progress", "usage"],
    ],
    [
      "not a payer",
      denied,
      ["blocked", "Only an Admin or a Billing person can pay", null],
    ],
  ];
  for (const [name, value, expected] of cases) {
    const replies = value === null ? [] : [response(value)];
    const { app } = await signedInWith("admin", ...replies, response(reviewed));
    const store = createReview(app, workspaceId, reviewed.choice.selection);
    if (value !== null) await usageStore(app, workspaceId).load();
    await store.ensure();
    assert.deepEqual(shape(summaryOf(app, store)), expected, name);
  }
});

test("a Viewer session keeps the payment closed, also beside a cached payer page", async () => {
  const { app, calls } = await signedInWith("viewer", response(usagePage()));
  await usageStore(app, workspaceId).load();
  await createPagePayment(app, workspaceId).submit({
    kind: "plan",
    review: reviewed,
  });
  assert.deepEqual(
    calls.map((call) => call.init.method),
    ["GET"],
  );
});
