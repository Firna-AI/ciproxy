import assert from "node:assert/strict";

import { parseCheckoutReview } from "../src/api/billing-parse.ts";
import type { PaidChangePayment } from "../src/api/billing-types.ts";
import { apiPrefix } from "../src/api/client.ts";
import { routePath } from "../src/api/route-path.ts";
import { billingHref, checkoutHref } from "../src/view/billing-routes.ts";
import { control, facts, visible } from "./browser-fixture.ts";
import { capturePage } from "./browser-pages-helpers.ts";
import { controlledBusinessReview } from "./browser-pages-paid-review.ts";
import type { PagesSmokeRun } from "./browser-pages-runtime.ts";
import { baseSelection } from "./fixtures-billing.ts";
import { workspaceId } from "./fixtures-core.ts";

export function hostedPaymentJourney(
  origin: string,
  output: string,
): PagesSmokeRun {
  return {
    name: "paid-change-hosted-confirmation",
    cleanup: () => control(origin, { pages: { held: [] } }),
    async check(page, _context, width) {
      await control(origin, { scenario: "m21-payment-required" });
      const reviewPath =
        apiPrefix +
        routePath("/workspaces/{workspace_id}/checkout/review", {
          workspace_id: workspaceId,
        });
      const reviewed = page.waitForResponse(
        (response) =>
          new URL(response.url()).pathname === reviewPath &&
          response.request().method() === "GET",
      );
      await page.goto(
        origin + checkoutHref(workspaceId, baseSelection("business")),
        { waitUntil: "domcontentloaded" },
      );
      const response = await reviewed;
      await response.finished();
      assert.equal(response.status(), 200);
      const initial = parseCheckoutReview(await response.json());
      assert.ok(initial?.quote, "A paid review must include its quote.");
      assert.deepEqual(initial.choice.selection, baseSelection("business"));
      const review = await controlledBusinessReview(page, origin, reviewPath);
      assert.ok(review.quote, "The final paid review must include its quote.");
      await page
        .getByRole("button", { name: "Move to Business", exact: true })
        .click();
      await visible(
        page.getByText("Your payment is not finished", { exact: true }),
      );
      await capturePage(page, output, "paid-change-bank-confirmation", width);
      const opened = page.waitForEvent("popup");
      await page
        .getByRole("link", { name: "Open the Stripe page", exact: true })
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
      const changePath = routePath(
        "/workspaces/{workspace_id}/billing/change",
        {
          workspace_id: workspaceId,
        },
      );
      const mutations = (await facts(origin)).filter(
        (fact) =>
          fact.method === "POST" && fact.path === `${apiPrefix}${changePath}`,
      );
      assert.equal(mutations.length, 1);
      assert.deepEqual(mutations[0]?.body, {
        quote_id: review.quote.id,
      } satisfies PaidChangePayment);
    },
  };
}
