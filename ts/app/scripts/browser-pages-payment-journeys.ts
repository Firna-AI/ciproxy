import assert from "node:assert/strict";

import {
  billingHref,
  checkoutHref,
  checkoutReturnHref,
  plansHref,
} from "../src/view/billing-routes.ts";
import { formatCents } from "../src/view/exact-format.ts";
import { control, facts, visible } from "./browser-fixture.ts";
import { absent, apiAction, capturePage } from "./browser-pages-helpers.ts";
import { fixtureReview, fixtureUsage } from "./browser-pages-billing-data.ts";
import { baseSelection } from "./fixtures-billing.ts";
import { fixtureCheckoutId } from "./fixtures-billing-payment.ts";
import { workspaceId } from "./fixtures-core.ts";
import type { PagesSmokeRun } from "./browser-pages-runtime.ts";

export function paymentJourneys(
  origin: string,
  output: string,
): PagesSmokeRun[] {
  return [
    {
      name: "offer-review-edit-payment-return",
      cleanup: () => control(origin, { pages: { held: [] } }),
      async check(page, _context, width) {
        await control(origin, { scenario: "m21-review-changed" });
        const usage = await fixtureUsage(origin);
        const selection = usage.offer.selection;
        await page.goto(origin + plansHref(workspaceId), {
          waitUntil: "domcontentloaded",
        });
        const choose = page
          .getByRole("region", { name: "Team", exact: true })
          .getByRole("link", { name: "Choose Team", exact: true });
        await visible(choose);
        const href = await choose.getAttribute("href");
        assert.equal(href, checkoutHref(workspaceId, selection));
        await choose.click();
        await visible(
          page.getByLabel(`Extra starts: ${selection.add_ons.start_blocks}`, {
            exact: true,
          }),
        );
        await visible(
          page.getByLabel(`Extra own machines: ${selection.add_ons.machines}`, {
            exact: true,
          }),
        );
        await visible(
          page.getByRole("button", {
            name: "Continue to payment",
            exact: true,
          }),
        );
        await capturePage(page, output, "review-offer-prefill", width);
        await control(origin, { pages: { held: ["checkout/review"] } });
        const requesting = page.waitForRequest(/\/checkout\/review\?/);
        await page
          .getByRole("button", { name: "More: Extra starts", exact: true })
          .click();
        await requesting;
        await visible(page.getByText("Pricing this choice", { exact: true }));
        await absent(
          page.getByRole("button", {
            name: "Continue to payment",
            exact: true,
          }),
        );
        await capturePage(page, output, "review-edit-pricing", width);
        const priced = page.waitForResponse(/\/checkout\/review\?/);
        await control(origin, { pages: { held: [] } });
        await (await priced).finished();
        const edited = {
          ...selection,
          add_ons: {
            ...selection.add_ons,
            start_blocks: selection.add_ons.start_blocks + 1,
          },
        };
        const reviewed = await fixtureReview(origin, edited);
        await visible(
          page
            .getByRole("region", { name: "Summary", exact: true })
            .getByText(
              `${formatCents(reviewed.choice.monthly_cents)} a month`,
              { exact: true },
            ),
        );
        await capturePage(page, output, "review-edited-choice", width);
        await page
          .getByRole("button", { name: "Continue to payment", exact: true })
          .click();
        await visible(
          page.getByRole("heading", {
            name: "Fixture navigation complete",
            exact: true,
          }),
        );
        assert.equal(
          new URL(page.url()).pathname,
          "/__fixture/stripe/checkout",
        );
        const payments = (await facts(origin)).filter(
          (fact) => fact.method === "POST" && fact.path.endsWith("/checkout"),
        );
        assert.equal(payments.length, 1);
        assert.deepEqual(payments[0]?.body, reviewed.choice.selection);
        await control(origin, { pages: { returned: "confirming" } });
        const exactRequest = page.waitForRequest(/\/checkout\/return$/);
        const first = page.waitForResponse(/\/checkout\/return$/);
        await page.goto(
          `${origin}/checkout-return?session_id=${fixtureCheckoutId}`,
          { waitUntil: "domcontentloaded" },
        );
        assert.deepEqual((await exactRequest).postDataJSON(), {
          session_id: fixtureCheckoutId,
        });
        await (await first).finished();
        await visible(
          page.getByRole("heading", {
            name: "Confirming your payment",
            exact: true,
          }),
        );
        assert.equal(
          new URL(page.url()).searchParams.get("workspace"),
          workspaceId,
        );
        await capturePage(page, output, "payment-return-context", width);
        for (let poll = 0; poll < 2; poll += 1)
          await apiAction(page, "checkout/return", "POST", () =>
            page.clock.runFor(3001),
          );
        await visible(
          page.getByRole("heading", {
            name: "Stripe is still confirming your payment",
            exact: true,
          }),
        );
        await capturePage(page, output, "payment-still-confirming", width);
        await control(origin, { pages: { returned: "confirmed" } });
        await apiAction(page, "checkout/return", "POST", () =>
          page.clock.runFor(3001),
        );
        await visible(
          page.getByRole("heading", { name: /Northstar is on Team/ }),
        );
        await page
          .getByRole("link", { name: "Open plan and usage", exact: true })
          .click();
        await visible(
          page.getByRole("heading", { name: "This period", exact: true }),
        );
      },
    },
    {
      name: "paid-change-hosted-confirmation",
      async check(page, _context, width) {
        await control(origin, { scenario: "m21-payment-required" });
        await page.goto(
          origin + checkoutHref(workspaceId, baseSelection("business")),
          { waitUntil: "domcontentloaded" },
        );
        await page
          .getByRole("button", { name: "Move to Business", exact: true })
          .click();
        await visible(
          page.getByText("Your bank needs to confirm this payment", {
            exact: true,
          }),
        );
        await capturePage(page, output, "paid-change-bank-confirmation", width);
        const opened = page.waitForEvent("popup");
        await page
          .getByRole("link", { name: "Confirm the payment", exact: true })
          .click();
        const payment = await opened;
        await visible(
          payment.getByRole("heading", {
            name: "Fixture navigation complete",
            exact: true,
          }),
        );
        assert.equal(
          new URL(payment.url()).pathname,
          "/__fixture/stripe/payment",
        );
        await payment.close();
        await page.goto(origin + billingHref(workspaceId), {
          waitUntil: "domcontentloaded",
        });
        await visible(
          page.getByRole("link", { name: "Continue payment", exact: true }),
        );
        await page.reload({ waitUntil: "domcontentloaded" });
        const resume = page.getByRole("link", {
          name: "Continue payment",
          exact: true,
        });
        await visible(resume);
        assert.equal(await resume.getAttribute("target"), "_blank");
        const continued = page.waitForEvent("popup");
        await resume.click();
        const bank = await continued;
        await visible(
          bank.getByRole("heading", {
            name: "Fixture navigation complete",
            exact: true,
          }),
        );
        assert.equal(new URL(bank.url()).pathname, "/__fixture/stripe/payment");
        await bank.close();
        const mutations = (await facts(origin)).filter(
          (fact) =>
            fact.method === "POST" && fact.path.endsWith("/billing/change"),
        );
        assert.equal(mutations.length, 1);
        assert.deepEqual(mutations[0]?.body, baseSelection("business"));
      },
    },
    {
      name: "return-confirmed-explicit-workspace",
      async check(page) {
        await control(origin, { scenario: "m21-payment-confirmed" });
        await page.goto(
          origin + checkoutReturnHref(workspaceId, fixtureCheckoutId),
          { waitUntil: "domcontentloaded" },
        );
        await visible(
          page.getByRole("heading", {
            name: "Northstar is on Team",
            exact: true,
          }),
        );
        assert.equal(
          new URL(page.url()).searchParams.get("workspace"),
          workspaceId,
        );
      },
    },
  ];
}
