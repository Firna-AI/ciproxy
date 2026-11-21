// Install actions keep native new tabs and pass through the document build gate.
import assert from "node:assert/strict";

import type { BrowserContext, Page } from "./browser-driver.ts";
import { control, facts, visible } from "./browser-fixture.ts";

export async function installNavigation(
  page: Page,
  context: BrowserContext,
  origin: string,
): Promise<void> {
  await control(origin, { scenario: "install-pending", reset: true });
  await page.goto(`${origin}/install`, { waitUntil: "domcontentloaded" });
  const link = page.getByRole("link", {
    name: "Install on another organization",
    exact: true,
  });
  await visible(link);
  const destination = "**/__fixture/github/install?attempt=fixture-attempt";
  const opened = Promise.race([
    context
      .waitForEvent("page")
      .then((popup) => ({ kind: "new-tab", popup }) as const),
    page.waitForURL(destination).then(() => ({ kind: "same-tab" }) as const),
  ]);
  await link.click({ modifiers: ["Control"] });
  const outcome = await opened;
  assert.equal(
    outcome.kind,
    "new-tab",
    "modified install must keep native navigation",
  );
  assert(outcome.kind === "new-tab");
  const install = outcome.popup.getByRole("link", {
    name: "Install on another organization",
    exact: true,
  });
  await visible(install);
  assert.equal(new URL(page.url()).pathname, "/install");
  assert.equal(
    (await facts(origin)).filter(
      (fact) => fact.path === "/api/installation-links/start",
    ).length,
    0,
  );
  await install.click();
  await outcome.popup.waitForURL(destination);
  assert.equal(
    (await facts(origin)).filter(
      (fact) =>
        fact.path === "/api/installation-links/start" &&
        fact.method === "POST" &&
        fact.status === 200,
    ).length,
    1,
  );
  await outcome.popup.close();

  await control(origin, { build: "fixture-build-b" });
  const refreshed = page.waitForResponse(/\/api\/session$/);
  await page.getByRole("button", { name: "Check again", exact: true }).click();
  const reply = await refreshed;
  await reply.finished();
  assert.equal(reply.headers()["x-ciproxy-build"], "fixture-build-b");
  const started = Promise.race([
    page.waitForURL(destination).then(() => "installed" as const),
    page.waitForResponse(/\/install$/).then(() => "reloaded" as const),
  ]);
  await link.click();
  assert.equal(
    await started,
    "installed",
    "an install action must reach POST before navigation",
  );
  assert.equal(
    (await facts(origin)).filter(
      (fact) =>
        fact.path === "/api/installation-links/start" &&
        fact.method === "POST" &&
        fact.status === 200,
    ).length,
    2,
  );
}
