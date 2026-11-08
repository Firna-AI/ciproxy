import assert from "node:assert/strict";

import type {
  CheckoutReturn,
  PaymentOutcome,
} from "../src/api/billing-types.ts";
import {
  billingPaths,
  checkoutReturnHref,
  sessionIdQuery,
} from "../src/view/billing-routes.ts";
import type {
  LivePagesCredentials,
  RuntimeRedirects,
} from "./live-pages-config.ts";

export function runtimeRedirects(
  config: LivePagesCredentials,
): RuntimeRedirects {
  const redirects = config.runtime_redirects;
  assert.notEqual(
    redirects,
    null,
    "Runtime redirect configuration is required.",
  );
  if (redirects === null) throw new Error("Runtime redirects are absent.");
  const success = new URL(redirects.success_url);
  assert.equal(success.origin, config.base_url);
  assert.equal(success.pathname, billingPaths.checkoutReturn);
  assert.equal(
    sessionIdQuery(success.searchParams.get("session_id") ?? undefined),
    "{CHECKOUT_SESSION_ID}",
  );
  const workspace = success.searchParams.get("workspace");
  if (workspace !== null) assert.equal(workspace, config.workspace_id);
  return redirects;
}

export function checkoutSession(
  outcome: PaymentOutcome,
): CheckoutReturn | null {
  if (outcome.state !== "open") return null;
  try {
    const url = new URL(outcome.url);
    if (url.protocol !== "https:" || url.hostname !== "checkout.stripe.com")
      return null;
    const id = url.pathname
      .split("/")
      .find((part) => /^cs_test_[a-zA-Z0-9_]+$/.test(part));
    return id === undefined ? null : { session_id: id };
  } catch {
    return null;
  }
}

export function privateReturnUrl(
  config: LivePagesCredentials,
  checkout: CheckoutReturn,
): string {
  return new URL(
    checkoutReturnHref(config.workspace_id, checkout.session_id),
    config.base_url,
  ).href;
}
