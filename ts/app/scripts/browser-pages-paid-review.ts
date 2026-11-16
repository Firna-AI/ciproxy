import assert from "node:assert/strict";

import { parseCheckoutReview } from "../src/api/billing-parse.ts";
import { formatCents } from "../src/view/exact-format.ts";
import type { Page } from "./browser-driver.ts";
import { control, visible } from "./browser-fixture.ts";
import { absent } from "./browser-pages-helpers.ts";
import { baseSelection } from "./fixtures-billing.ts";

export async function controlledBusinessReview(
  page: Page,
  origin: string,
  reviewPath: string,
) {
  const summary = page.getByRole("region", { name: "Summary", exact: true });
  const pay = summary.getByRole("button", {
    name: "Move to Business",
    exact: true,
  });
  await visible(pay);
  assert.equal(await pay.isEnabled(), true);
  const increased = page.waitForResponse((response) => {
    const url = new URL(response.url());
    return (
      url.pathname === reviewPath &&
      response.request().method() === "GET" &&
      url.searchParams.get("tier") === "business" &&
      url.searchParams.get("start_blocks") === "1" &&
      url.searchParams.get("machines") === "0"
    );
  });
  await page
    .getByRole("button", { name: "More: Extra starts", exact: true })
    .click();
  const moreResponse = await increased;
  await moreResponse.finished();
  assert.equal(moreResponse.status(), 200);
  const more = parseCheckoutReview(await moreResponse.json());
  assert.ok(more?.quote, "The edited paid review must include its quote.");
  assert.deepEqual(more.choice.selection, {
    tier: "business",
    add_ons: { start_blocks: 1, machines: 0 },
  });
  await visible(page.getByLabel("Extra starts: 1", { exact: true }));
  await visible(
    summary.getByText(formatCents(more.today_cents), { exact: true }),
  );

  await control(origin, { pages: { held: ["checkout/review"] } });
  const requested = page.waitForRequest(new RegExp(`${reviewPath}[?]`));
  await page
    .getByRole("button", { name: "Fewer: Extra starts", exact: true })
    .click();
  const request = await requested;
  assert.equal(request.method(), "GET");
  const query = new URL(request.url()).searchParams;
  assert.deepEqual(query.getAll("tier"), ["business"]);
  assert.deepEqual(query.getAll("start_blocks"), ["0"]);
  assert.deepEqual(query.getAll("machines"), ["0"]);
  const priced = page.waitForResponse(
    (response) => response.request() === request,
  );
  await visible(page.getByText("Pricing this choice", { exact: true }));
  await absent(pay);
  await control(origin, { pages: { held: [] } });
  const response = await priced;
  await response.finished();
  assert.equal(response.status(), 200);
  const review = parseCheckoutReview(await response.json());
  assert.ok(review?.quote, "The final paid review must include its quote.");
  assert.deepEqual(review.choice.selection, baseSelection("business"));
  await visible(page.getByLabel("Extra starts: 0", { exact: true }));
  await visible(
    summary.getByText(formatCents(review.today_cents), { exact: true }),
  );
  await visible(pay);
  assert.equal(await pay.isEnabled(), true);
  return review;
}
