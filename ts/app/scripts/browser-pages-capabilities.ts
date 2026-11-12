import assert from "node:assert/strict";

import {
  openPaymentKey,
  type OpenPayment,
} from "../src/state/billing-open-payment.ts";
import { billingHref, plansHref } from "../src/view/billing-routes.ts";
import { control, facts, visible } from "./browser-fixture.ts";
import { absent, capturePage } from "./browser-pages-helpers.ts";
import { fixtureCheckoutId } from "./fixtures-billing-payment.ts";
import { workspaceId } from "./fixtures-core.ts";
import type { PagesSmokeRun } from "./browser-pages-runtime.ts";

export function capabilityRuns(
  origin: string,
  output: string,
): PagesSmokeRun[] {
  const saved: OpenPayment = {
    kind: "checkout",
    url: `/__fixture/stripe/checkout?workspace=${workspaceId}&session_id=${fixtureCheckoutId}`,
  };
  return (
    [
      ["usage", billingHref(workspaceId), "This period"],
      ["plans", plansHref(workspaceId), "Business"],
    ] as const
  ).map(([name, path, heading]) => ({
    name: `viewer-saved-checkout-${name}`,
    // Arrange a URL kept from an earlier payer role in this same tab.
    init: `sessionStorage.setItem(${JSON.stringify(openPaymentKey(workspaceId))}, ${JSON.stringify(JSON.stringify(saved))});`,
    async check(page, _context, width) {
      await control(origin, {
        scenario: "m21-usage-pending",
        pages: { role: "viewer" },
      });
      await page.goto(origin + path, { waitUntil: "domcontentloaded" });
      await visible(page.getByRole("heading", { name: heading, exact: true }));
      await capturePage(page, output, `viewer-saved-url-${name}`, width);
      await absent(
        page.getByRole("link", { name: "Continue payment", exact: true }),
      );
      await absent(
        page.getByRole("button", { name: "Invoices and card", exact: true }),
      );
      await absent(
        page.getByRole("button", { name: "Save add-ons", exact: true }),
      );
      assert.equal(
        (await facts(origin)).filter(
          (fact) => fact.method !== "GET" && fact.kind === "api",
        ).length,
        0,
      );
    },
  }));
}
