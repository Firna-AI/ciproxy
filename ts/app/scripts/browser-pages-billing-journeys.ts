import assert from "node:assert/strict";

import {
  billingHref,
  checkoutHref,
  plansHref,
} from "../src/view/billing-routes.ts";
import { formatCents } from "../src/view/exact-format.ts";
import { control, facts, visible } from "./browser-fixture.ts";
import { apiAction, capturePage } from "./browser-pages-helpers.ts";
import { fixtureUsage } from "./browser-pages-billing-data.ts";
import { baseSelection } from "./fixtures-billing.ts";
import { workspaceId } from "./fixtures-core.ts";
import type { PagesSmokeRun } from "./browser-pages-runtime.ts";

export function billingJourneys(
  origin: string,
  output: string,
): PagesSmokeRun[] {
  return [
    {
      name: "authoritative-add-on-save",
      async check(page, _context, width) {
        await control(origin, { scenario: "m21-usage-soft-starts" });
        const usage = await fixtureUsage(origin);
        await page.goto(origin + billingHref(workspaceId), {
          waitUntil: "domcontentloaded",
        });
        await page
          .getByRole("button", {
            name: "Use the add-ons that cover your use",
            exact: true,
          })
          .click();
        await visible(
          page.getByLabel(
            `Extra starts: ${usage.offer.selection.add_ons.start_blocks}`,
            { exact: true },
          ),
        );
        await visible(
          page
            .getByRole("region", { name: "Add-ons", exact: true })
            .getByText(`${formatCents(usage.offer.monthly_cents)} a month`, {
              exact: true,
            }),
        );
        await capturePage(page, output, "add-ons-authoritative-offer", width);
        await apiAction(page, "billing/add-ons", "PUT", () =>
          page
            .getByRole("button", { name: "Save add-ons", exact: true })
            .click(),
        );
        await visible(
          page.getByRole("heading", {
            name: "Your add-ons are saved",
            exact: true,
          }),
        );
        const saved = (await facts(origin)).filter(
          (fact) =>
            fact.method === "PUT" && fact.path.endsWith("/billing/add-ons"),
        );
        assert.equal(saved.length, 1);
        assert.deepEqual(saved[0]?.body, usage.offer.selection.add_ons);
      },
    },
    {
      name: "unpriced-add-on-cannot-save",
      async check(page) {
        await control(origin, { scenario: "m21-usage-paid" });
        await page.goto(origin + billingHref(workspaceId), {
          waitUntil: "domcontentloaded",
        });
        await page
          .getByRole("button", { name: "More: Extra starts", exact: true })
          .click();
        await visible(
          page.getByText("Monthly total not available", { exact: true }),
        );
        assert.equal(
          await page
            .getByRole("button", { name: "Save add-ons", exact: true })
            .isEnabled(),
          false,
        );
        assert.equal(
          (await facts(origin)).filter((fact) =>
            fact.path.endsWith("/checkout/review"),
          ).length,
          0,
        );
      },
    },
    {
      name: "portal-navigation-and-return-context",
      async check(page) {
        await control(origin, { scenario: "m21-billing" });
        await page.goto(origin + billingHref(workspaceId), {
          waitUntil: "domcontentloaded",
        });
        await page
          .getByRole("button", { name: "Invoices and card", exact: true })
          .click();
        await visible(
          page.getByRole("heading", {
            name: "Fixture navigation complete",
            exact: true,
          }),
        );
        assert.equal(new URL(page.url()).pathname, "/__fixture/stripe/portal");
        await page.goto(origin + "/billing", { waitUntil: "domcontentloaded" });
        await visible(
          page.getByRole("heading", { name: "This period", exact: true }),
        );
        assert.equal(
          new URL(page.url()).searchParams.get("workspace"),
          workspaceId,
        );
        assert.equal(
          (await facts(origin)).filter(
            (fact) =>
              fact.method === "POST" && fact.path.endsWith("/billing/portal"),
          ).length,
          1,
        );
      },
    },
    {
      name: "move-to-free-confirmation",
      async check(page, _context, width) {
        await control(origin, { scenario: "m21-move-to-free" });
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
        await capturePage(page, output, "move-to-free-dialog", width);
        assert.equal(
          (await facts(origin)).filter((fact) =>
            fact.path.endsWith("/billing/move-to-free"),
          ).length,
          0,
        );
        await dialog
          .getByRole("button", { name: "Keep Team", exact: true })
          .click();
        await dialog.waitFor({ state: "hidden" });
        await page
          .getByRole("button", { name: "Move to Free", exact: true })
          .click();
        await dialog
          .getByRole("button", { name: "Move to Free", exact: true })
          .click();
        await visible(
          page.getByRole("heading", { name: "This period", exact: true }),
        );
        await visible(
          page.getByText("Team ends on 3 December", { exact: true }),
        );
        assert.equal(
          (await facts(origin)).filter(
            (fact) =>
              fact.method === "POST" &&
              fact.path.endsWith("/billing/move-to-free"),
          ).length,
          1,
        );
      },
    },
    {
      name: "checkout-cancel-reload-continues-same-payment",
      async check(page, _context, width) {
        await control(origin, { scenario: "m21-review-free" });
        await page.goto(
          origin + checkoutHref(workspaceId, baseSelection("team")),
          { waitUntil: "domcontentloaded" },
        );
        await page
          .getByRole("button", { name: "Continue to payment", exact: true })
          .click();
        await visible(
          page.getByRole("heading", {
            name: "Fixture navigation complete",
            exact: true,
          }),
        );
        const provider = page.url();
        await page.goto(origin + "/plans", { waitUntil: "domcontentloaded" });
        const resume = page.getByRole("link", {
          name: "Continue payment",
          exact: true,
        });
        await visible(resume);
        await page.reload({ waitUntil: "domcontentloaded" });
        await visible(resume);
        assert.equal(
          new URL((await resume.getAttribute("href")) ?? "", origin).href,
          provider,
        );
        assert.equal(
          await page
            .getByRole("button", { name: "Choose Team", exact: true })
            .isEnabled(),
          false,
        );
        assert.equal(
          await page
            .getByRole("button", { name: "Choose Business", exact: true })
            .isEnabled(),
          false,
        );
        await capturePage(page, output, "checkout-cancel-reload", width);
        await resume.click();
        await visible(
          page.getByRole("heading", {
            name: "Fixture navigation complete",
            exact: true,
          }),
        );
        assert.equal(page.url(), provider);
        assert.equal(
          (await facts(origin)).filter(
            (fact) => fact.method === "POST" && fact.path.endsWith("/checkout"),
          ).length,
          1,
        );
      },
    },
  ];
}
