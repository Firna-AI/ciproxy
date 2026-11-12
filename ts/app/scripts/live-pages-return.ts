import assert from "node:assert/strict";

import { createBillingEndpoints } from "../src/api/billing.ts";
import { sameSelection } from "../src/api/billing-payment.ts";
import { failureStatus, type ApiClient } from "../src/api/client.ts";
import { apiValue, type LiveChecks } from "./live-pages-checks.ts";
import type { LivePagesCredentials } from "./live-pages-config.ts";
import { readPrivateJson, writePrivateJson } from "./live-pages-private.ts";
import {
  readCheckoutRecord,
  type CheckoutRecord,
} from "./live-pages-record.ts";
import { privateReturnUrl } from "./live-pages-redirects.ts";

export async function checkLiveReturn(
  api: ApiClient,
  config: LivePagesCredentials,
  checks: LiveChecks,
): Promise<void> {
  const path = config.result_path;
  const saved = await checks.check("exact saved Checkout context", async () => {
    assert.notEqual(path, null, "A private result path is required.");
    if (path === null) throw new Error("A private result path is required.");
    const record = readCheckoutRecord(await readPrivateJson(path));
    assert.equal(record.base_url, config.base_url);
    assert.equal(record.workspace_id, config.workspace_id);
    assert.notEqual(record.phase, "reviewed", "Checkout has not opened.");
    return record;
  });
  if (
    path === null ||
    saved.phase === "reviewed" ||
    saved.checkout === null ||
    saved.return_url === null
  ) {
    throw new Error("The saved Checkout has no exact session.");
  }
  const checkout = saved.checkout;
  await checks.check("explicit workspace return route", () => {
    assert.equal(saved.return_url, privateReturnUrl(config, checkout));
  });
  const billing = createBillingEndpoints(api);
  const answer = await billing.checkoutReturn(
    config.workspace_id,
    checkout.session_id,
  );
  if (!answer.ok && failureStatus(answer.failure) === 409) {
    checks.pending("Checkout return reconciliation busy");
    return;
  }
  const returned = await checks.check("exact-session Checkout return", () =>
    apiValue(answer),
  );
  const after = await checks.check(
    "current plan after Checkout return",
    async () => apiValue(await billing.usage(config.workspace_id)),
  );
  const record: CheckoutRecord = {
    ...saved,
    phase: "returned",
    checkout,
    return_url: saved.return_url,
    returned,
    after,
  };
  await checks.check("private Checkout result updated", () =>
    writePrivateJson(path, record, false),
  );
  const outcome = returned.outcome;
  switch (outcome.state) {
    case "confirmed":
      await checks.check("confirmed reviewed plan and add-ons", () => {
        assert.equal(
          sameSelection(after.status.selection, saved.review.choice.selection),
          true,
        );
        assert.equal(
          after.current.monthly_cents,
          saved.review.choice.monthly_cents,
        );
        assert.equal(after.status.subscription, "active");
        assert.equal(after.status.payment_pending, false);
      });
      return;
    case "confirming":
      checks.pending("Checkout payment confirmation");
      return;
    case "open":
      checks.pending("Checkout payment needs completion");
      return;
    case "unfinished":
      checks.fail("Checkout expired without payment");
      return;
    case "payment_required":
      checks.fail("Unexpected Checkout payment-required outcome");
      return;
  }
  const handled: never = outcome;
  return handled;
}
