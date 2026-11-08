import {
  billingHref,
  checkoutHref,
  plansHref,
} from "../src/view/billing-routes.ts";
import { control, visible } from "./browser-fixture.ts";
import { capturePage } from "./browser-pages-helpers.ts";
import { baseSelection } from "./fixtures-billing.ts";
import { workspaceId } from "./fixtures-core.ts";
import type { PagesSmokeRun } from "./browser-pages-runtime.ts";

export function billingErrorRuns(
  origin: string,
  output: string,
): PagesSmokeRun[] {
  return [
    {
      name: "portal-error-state",
      expectedHttpStatuses: [503],
      async check(page) {
        await control(origin, { scenario: "m21-portal-error" });
        await page.goto(origin + billingHref(workspaceId), {
          waitUntil: "domcontentloaded",
        });
        await page
          .getByRole("button", { name: "Invoices and card", exact: true })
          .click();
        await visible(
          page.getByText(
            "Unable to open the Stripe billing portal. Try again.",
            { exact: true },
          ),
        );
      },
    },
    {
      name: "move-to-free-error-state",
      expectedHttpStatuses: [503],
      async check(page) {
        await control(origin, { scenario: "m21-move-to-free-error" });
        await page.goto(origin + plansHref(workspaceId), {
          waitUntil: "domcontentloaded",
        });
        await page
          .getByRole("button", { name: "Move to Free", exact: true })
          .click();
        const dialog = page.getByRole("dialog", {
          name: "Move Northstar to Free?",
          exact: true,
        });
        await visible(dialog);
        await dialog
          .getByRole("button", { name: "Move to Free", exact: true })
          .click();
        await visible(
          dialog.getByText("Move to Free did not finish", { exact: true }),
        );
      },
    },
    ...(
      [
        ["payment-request-error", "open", "The payment did not start"],
        ["payment-unfinished-state", "unfinished", "Your plan did not change"],
        [
          "payment-confirming-state",
          "confirming",
          "Stripe is confirming your payment",
        ],
      ] as const
    ).map(([name, payment, title]): PagesSmokeRun => ({
      name,
      expectedHttpStatuses:
        name === "payment-request-error" ? [503] : undefined,
      async check(page, _context, width) {
        await control(origin, {
          scenario:
            name === "payment-request-error"
              ? "m21-payment-error"
              : "m21-review-free",
          pages: { payment },
        });
        await page.goto(
          origin + checkoutHref(workspaceId, baseSelection("team")),
          { waitUntil: "domcontentloaded" },
        );
        await page
          .getByRole("button", { name: "Continue to payment", exact: true })
          .click();
        await visible(page.getByText(title, { exact: true }));
        await capturePage(page, output, name, width);
      },
    })),
    {
      name: "paid-change-confirmed",
      async check(page) {
        await control(origin, { scenario: "m21-review-paid" });
        await page.goto(
          origin + checkoutHref(workspaceId, baseSelection("business")),
          { waitUntil: "domcontentloaded" },
        );
        await visible(
          page
            .getByRole("region", { name: "Summary", exact: true })
            .getByText(/ends on 3\s+December/),
        );
        await page
          .getByRole("button", { name: "Move to Business", exact: true })
          .click();
        await visible(
          page.getByRole("heading", {
            name: "Northstar is on Business",
            exact: true,
          }),
        );
      },
    },
  ];
}
