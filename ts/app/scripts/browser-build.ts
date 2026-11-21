import assert from "node:assert/strict";

import type { Page } from "./browser-driver.ts";
import {
  control,
  documentReloaded,
  facts,
  visible,
} from "./browser-fixture.ts";
import { otherWorkspaceId, workspaceId } from "./fixtures-core.ts";

export async function buildNavigation(
  page: Page,
  origin: string,
  mode: "link" | "programmatic" | "history",
): Promise<void> {
  await control(origin, { scenario: "admin" });
  await page.goto(`${origin}/jobs?workspace=${workspaceId}`, {
    waitUntil: "domcontentloaded",
  });
  await page.getByRole("link", { name: /^(Set up|Finish setup)/ }).click();
  await visible(page.getByTestId("install-command"));
  const current = page.url();
  await control(origin, { build: "fixture-build-b" });
  const buildReply = page.waitForResponse(/\/api\/workspaces\/[^/]+\/setup$/);
  await page.clock.runFor(5001);
  const reply = await buildReply;
  await reply.finished();
  await page.clock.runFor(1);
  assert.equal(reply.headers()["x-ciproxy-build"], "fixture-build-b");
  assert.equal(
    page.url(),
    current,
    "A build change must preserve the current page",
  );
  const before = (await facts(origin)).length;
  if (mode === "link") {
    await page.getByRole("link", { name: "Machines", exact: true }).click();
    await visible(page.getByRole("heading", { name: "Machines", exact: true }));
    await documentReloaded(origin, before, "/machines");
  } else if (mode === "programmatic") {
    await page
      .getByRole("button", {
        name: "Workspace Northstar. Switch workspace",
        exact: true,
      })
      .click();
    await page.getByRole("menuitem", { name: "Orbit", exact: true }).click();
    await visible(
      page.getByRole("heading", {
        name: "Waiting for an Admin of Orbit",
        exact: true,
      }),
    );
    assert.ok(page.url().includes(otherWorkspaceId));
    await documentReloaded(origin, before, "/");
  } else {
    await page.goBack({ waitUntil: "domcontentloaded" });
    await visible(page.getByRole("heading", { name: "Jobs", exact: true }));
    await documentReloaded(origin, before, "/jobs");
  }
}
