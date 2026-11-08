import assert from "node:assert/strict";
import test from "node:test";

import type { PaymentOutcome } from "../src/api/billing-types.ts";
import {
  createAppServices,
  type AppServices,
} from "../src/services/services.ts";
import { usageStore } from "../src/state/billing.ts";
import {
  continuablePayment,
  createCheckoutReturn,
  createPagePayment,
} from "../src/state/billing-flows.ts";
import { session, workspaceId } from "../scripts/fixtures-core.ts";
import {
  offer,
  pendingUsagePage,
  review,
  selection,
  signedInWith,
  teamOffer,
  unavailable,
  usagePage,
} from "./billing-support.ts";
import { response, settled, transportQueue, type Reply } from "./support.ts";

const checkoutUrl = "https://checkout.stripe.test/c/pay/cs_test_open";
const bankUrl = "https://invoice.stripe.test/i/in_test_bank";

/**
 * The same tab after a reload: new services over the same tab storage, which
 * read the session of a payer again. `calls` leaves that session read out.
 */
async function reload(app: AppServices, ...replies: Reply[]) {
  const queue = transportQueue(response(session()), ...replies);
  const services = createAppServices({
    ...app.env,
    transport: queue.transport,
  });
  await services.session.ensure();
  queue.calls.splice(0, queue.calls.length);
  return { app: services, calls: queue.calls };
}

async function offered(app: AppServices) {
  const store = usageStore(app, workspaceId);
  await store.load();
  return continuablePayment(
    app,
    workspaceId,
    store.state.get(),
    (page) => page.status,
  );
}

function teamUsage() {
  const page = usagePage({ current: teamOffer, crossing_day: null });
  page.status.selection = selection("team");
  return page;
}

test("a cancelled Checkout continues after a reload with no second payment request", async () => {
  const first = await signedInWith(
    "admin",
    response(usagePage()),
    response({ state: "open", url: checkoutUrl } satisfies PaymentOutcome),
  );
  await usageStore(first.app, workspaceId).load();
  await createPagePayment(first.app, workspaceId).submit({
    kind: "plan",
    review: review(),
  });
  assert.deepEqual(first.documents, [checkoutUrl]);
  // Stripe's cancel URL brings the person back; the tab reloads.
  const back = await reload(first.app, response(pendingUsagePage()));
  assert.deepEqual(await offered(back.app), {
    url: checkoutUrl,
    kind: "checkout",
  });
  await createPagePayment(back.app, workspaceId).submit({
    kind: "plan",
    review: review(),
  });
  assert.deepEqual(
    back.calls.map((call) => `${call.init.method} ${call.url}`),
    [`GET /api/workspaces/${workspaceId}/billing`],
  );
});

test("a bank confirmation continues after a reload", async () => {
  const pending = pendingUsagePage();
  pending.status.selection = selection("team");
  const first = await signedInWith(
    "admin",
    response(teamUsage()),
    response({
      state: "payment_required",
      payment_url: bankUrl,
    } satisfies PaymentOutcome),
    response(pending),
  );
  await usageStore(first.app, workspaceId).load();
  const business = offer(selection("business"), "29900", "150000", "10");
  await createPagePayment(first.app, workspaceId).submit({
    kind: "plan",
    review: review(business, { prorated: true }),
  });
  assert.equal(
    first.calls[1]?.url,
    `/api/workspaces/${workspaceId}/billing/change`,
  );
  const back = await reload(first.app, response(pending));
  assert.deepEqual(await offered(back.app), { url: bankUrl, kind: "bank" });
  assert.equal(back.calls.length, 1);
});

test("a failed refresh offers no saved payment", async () => {
  const first = await signedInWith(
    "admin",
    response(usagePage()),
    response({ state: "open", url: checkoutUrl } satisfies PaymentOutcome),
  );
  await usageStore(first.app, workspaceId).load();
  await createPagePayment(first.app, workspaceId).submit({
    kind: "plan",
    review: review(),
  });
  const back = await reload(
    first.app,
    response(pendingUsagePage()),
    response(unavailable, 503),
  );
  assert.notEqual(await offered(back.app), null);
  assert.equal(await offered(back.app), null);
});

test("a confirmed return ends the saved payment, also when the read after it fails", async () => {
  const first = await signedInWith(
    "admin",
    response(usagePage()),
    response({ state: "open", url: checkoutUrl } satisfies PaymentOutcome),
  );
  await usageStore(first.app, workspaceId).load();
  await createPagePayment(first.app, workspaceId).submit({
    kind: "plan",
    review: review(),
  });
  const done = await reload(
    first.app,
    response({ outcome: { state: "confirmed" }, banner: null }),
    response(unavailable, 503),
    response(pendingUsagePage()),
  );
  const ret = createCheckoutReturn(done.app, workspaceId, "cs_test_open");
  ret.start();
  await settled();
  await settled();
  ret.stop();
  assert.equal(await offered(done.app), null);
});

test("a read with no pending payment ends the saved payment", async () => {
  const first = await signedInWith(
    "admin",
    response(usagePage()),
    response({ state: "open", url: checkoutUrl } satisfies PaymentOutcome),
  );
  await usageStore(first.app, workspaceId).load();
  await createPagePayment(first.app, workspaceId).submit({
    kind: "plan",
    review: review(),
  });
  const back = await reload(
    first.app,
    response(usagePage()),
    response(pendingUsagePage()),
  );
  assert.equal(await offered(back.app), null);
  assert.equal(await offered(back.app), null);
});

test("a person who cannot pay is offered no saved payment", async () => {
  const first = await signedInWith(
    "admin",
    response(usagePage()),
    response({ state: "open", url: checkoutUrl } satisfies PaymentOutcome),
  );
  await usageStore(first.app, workspaceId).load();
  await createPagePayment(first.app, workspaceId).submit({
    kind: "plan",
    review: review(),
  });
  // The role of the person in this tab ends; the saved URL stays in the tab.
  const viewer = pendingUsagePage();
  viewer.status.can_pay = false;
  const back = await reload(first.app, response(viewer));
  assert.equal(await offered(back.app), null);
});
