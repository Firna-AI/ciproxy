import assert from "node:assert/strict";

import { createBillingEndpoints, reviewRequest } from "../src/api/billing.ts";
import { sameSelection, submitPayment } from "../src/api/billing-payment.ts";
import type { ApiClient } from "../src/api/client.ts";
import { apiValue, type LiveCheck } from "./live-pages-checks.ts";
import type { LivePagesCredentials } from "./live-pages-config.ts";
import { writePrivateJson } from "./live-pages-private.ts";
import type { CheckoutRecord } from "./live-pages-record.ts";
import {
  checkoutSession,
  privateReturnUrl,
  runtimeRedirects,
} from "./live-pages-redirects.ts";

export async function startLiveCheckout(
  api: ApiClient,
  config: LivePagesCredentials,
  check: LiveCheck,
): Promise<void> {
  const path = config.result_path;
  const redirects = await check(
    "private result and runtime redirect configuration",
    () => {
      assert.notEqual(path, null, "A private result path is required.");
      return runtimeRedirects(config);
    },
  );
  if (path === null) throw new Error("A private result path is required.");
  const billing = createBillingEndpoints(api);
  const before = await check("Free workspace can start Checkout", async () => {
    const usage = apiValue(await billing.usage(config.workspace_id));
    assert.equal(usage.status.selection.tier, "free");
    assert.equal(usage.status.can_pay, true);
    assert.equal(usage.status.payment_pending, false);
    assert.equal(usage.status.operation_in_progress, false);
    return usage;
  });
  const selection = config.selection ?? before.offer.selection;
  const review = await check("authoritative Checkout review", async () => {
    const reviewed = apiValue(
      await billing.review(config.workspace_id, reviewRequest(selection)),
    );
    assert.equal(reviewed.prorated, false);
    assert.equal(sameSelection(reviewed.choice.selection, selection), true);
    return reviewed;
  });
  const prepared: CheckoutRecord = {
    version: 1,
    phase: "reviewed",
    base_url: config.base_url,
    workspace_id: config.workspace_id,
    created_at: new Date().toISOString(),
    before,
    review,
    runtime_redirects: redirects,
  };
  await check("reserve one private Checkout result", () =>
    writePrivateJson(path, prepared, true),
  );
  await check("one Stripe test Checkout opened", async () => {
    const payment = apiValue(
      await submitPayment(api, config.workspace_id, { kind: "plan", review }),
    );
    const checkout = checkoutSession(payment);
    const submitted: CheckoutRecord = {
      ...prepared,
      phase: "submitted",
      payment,
      checkout,
      return_url: checkout === null ? null : privateReturnUrl(config, checkout),
    };
    await writePrivateJson(path, submitted, false);
    assert.equal(payment.state, "open");
    assert.notEqual(
      checkout,
      null,
      "The provider must return a Stripe test Checkout session.",
    );
  });
  await check("Checkout keeps the current plan until payment", async () => {
    const usage = apiValue(await billing.usage(config.workspace_id));
    assert.equal(
      sameSelection(usage.status.selection, before.status.selection),
      true,
    );
    assert.equal(usage.status.payment_pending, true);
  });
}
