import assert from "node:assert/strict";

import {
  parseCheckoutReview,
  parsePaymentOutcome,
  parsePlanUsagePage,
} from "../src/api/billing-parse.ts";
import type { PaidChangePayment } from "../src/api/billing-types.ts";
import { apiPrefix } from "../src/api/client.ts";
import { parseProblem } from "../src/api/parse-session.ts";
import { routePath } from "../src/api/route-path.ts";
import { checkoutHref } from "../src/view/billing-routes.ts";
import { formatCents } from "../src/view/exact-format.ts";
import type { BrowserResponse } from "./browser-driver.ts";
import { control, facts, visible } from "./browser-fixture.ts";
import { absent, capturePage } from "./browser-pages-helpers.ts";
import type { PagesSmokeRun } from "./browser-pages-runtime.ts";
import { baseSelection } from "./fixtures-billing.ts";
import { workspaceId } from "./fixtures-core.ts";

export function quoteJourney(origin: string, output: string): PagesSmokeRun {
  const params = { workspace_id: workspaceId };
  const reviewPath = `${apiPrefix}${routePath("/workspaces/{workspace_id}/checkout/review", params)}`;
  const billingPath = `${apiPrefix}${routePath("/workspaces/{workspace_id}/billing", params)}`;
  const changePath = `${apiPrefix}${routePath("/workspaces/{workspace_id}/billing/change", params)}`;
  const responseFor =
    (path: string, method: string) => (response: BrowserResponse) =>
      new URL(response.url()).pathname === path &&
      response.request().method() === method;
  const payments = async () =>
    (await facts(origin)).filter(
      (fact) => fact.method === "POST" && fact.path === changePath,
    );
  return {
    name: "paid-change-refusal-review-again-explicit-payment",
    expectedHttpStatuses: [409],
    cleanup: () =>
      control(origin, { pages: { held: [], reviewRequired: false } }),
    async check(page, _context, width) {
      await control(origin, { scenario: "m21-review-paid" });
      const firstReview = page.waitForResponse(responseFor(reviewPath, "GET"));
      await page.goto(
        origin + checkoutHref(workspaceId, baseSelection("business")),
        { waitUntil: "domcontentloaded" },
      );
      const firstResponse = await firstReview;
      await firstResponse.finished();
      assert.equal(firstResponse.status(), 200);
      const first = parseCheckoutReview(await firstResponse.json());
      assert.ok(first?.quote, "The paid review must have a quote.");
      const summary = page.getByRole("region", {
        name: "Summary",
        exact: true,
      });
      const pay = summary.getByRole("button", {
        name: "Move to Business",
        exact: true,
      });
      const again = summary.getByRole("button", {
        name: "Review again",
        exact: true,
      });
      await visible(pay);
      assert.equal(await pay.isEnabled(), true);
      await control(origin, {
        pages: { held: ["billing"], reviewRequired: true },
      });
      const refused = page.waitForResponse(responseFor(changePath, "POST"));
      const requested = page.waitForRequest(new RegExp(`${billingPath}$`));
      await pay.click();
      const refusal = await refused;
      await refusal.finished();
      assert.equal(refusal.status(), 409);
      assert.equal(parseProblem(await refusal.json())?.code, "review_required");
      const billingRequest = await requested;
      const settled = page.waitForResponse(
        (response) => response.request() === billingRequest,
      );
      await absent(again);
      await absent(
        summary.getByText("This review is out of date", { exact: true }),
      );
      const firstPayments = await payments();
      assert.equal(firstPayments.length, 1);
      assert.equal(firstPayments[0]?.status, 409);
      assert.deepEqual(firstPayments[0]?.body, {
        quote_id: first.quote.id,
      } satisfies PaidChangePayment);
      await control(origin, { pages: { held: [] } });
      const freshResponse = await settled;
      await freshResponse.finished();
      assert.equal(freshResponse.status(), 200);
      const fresh = parsePlanUsagePage(await freshResponse.json());
      assert.equal(fresh?.status.selection.tier, "team");
      assert.equal(fresh?.status.payment_pending, false);
      assert.equal(fresh?.status.operation_in_progress, false);
      await visible(
        summary.getByText("This review is out of date", { exact: true }),
      );
      await visible(
        summary.getByText(
          "The payment did not start. Review the change again to see its current charge.",
          { exact: true },
        ),
      );
      await visible(again);
      await absent(pay);
      await absent(summary.getByText("Charged today", { exact: true }));
      await capturePage(page, output, "paid-change-quote-refused", width);
      await control(origin, { pages: { reviewRequired: false } });
      const newReview = page.waitForResponse(responseFor(reviewPath, "GET"));
      await again.click();
      const newResponse = await newReview;
      await newResponse.finished();
      assert.equal(newResponse.status(), 200);
      const next = parseCheckoutReview(await newResponse.json());
      assert.ok(next?.quote, "Review again must return a quote.");
      assert.notEqual(next.quote.id, first.quote.id);
      assert.deepEqual(next.choice, first.choice);
      assert.equal(next.today_cents, first.today_cents);
      await visible(pay);
      assert.equal(await pay.isEnabled(), true);
      await visible(
        summary.getByText(formatCents(next.today_cents), { exact: true }),
      );
      await absent(again);
      assert.equal(
        (await payments()).length,
        1,
        "Review again must not start payment.",
      );
      await capturePage(page, output, "paid-change-reviewed-again", width);
      const paid = page.waitForResponse(responseFor(changePath, "POST"));
      await pay.click();
      const paymentResponse = await paid;
      await paymentResponse.finished();
      assert.equal(paymentResponse.status(), 200);
      assert.deepEqual(parsePaymentOutcome(await paymentResponse.json()), {
        state: "confirmed",
      });
      await visible(
        page.getByRole("heading", {
          name: "Northstar is on Business",
          exact: true,
        }),
      );
      const completed = await payments();
      assert.equal(completed.length, 2);
      assert.deepEqual(completed[1]?.body, {
        quote_id: next.quote.id,
      } satisfies PaidChangePayment);
      const requests = (await facts(origin)).filter(
        (fact) => fact.kind === "api",
      );
      const firstPayment = requests.findIndex(
        (fact) => fact.path === changePath,
      );
      assert.deepEqual(
        requests
          .slice(firstPayment, firstPayment + 4)
          .map(({ method, path }) => ({ method, path })),
        [
          { method: "POST", path: changePath },
          { method: "GET", path: billingPath },
          { method: "GET", path: reviewPath },
          { method: "POST", path: changePath },
        ],
      );
    },
  };
}
