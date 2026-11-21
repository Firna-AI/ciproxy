import assert from "node:assert/strict";

import type { components } from "../../api/index.ts";
import { workspaceHref } from "../src/routing/pages.ts";
import { launchBrowser, type Page } from "./browser-driver.ts";
import { visible } from "./browser-fixture.ts";
import type { DashboardCredentials } from "./m20-live-credentials.ts";

export type LiveCheck = (
  name: string,
  run: () => Promise<void>,
) => Promise<void>;

async function reload(page: Page, heading: string): Promise<void> {
  await visible(page.getByRole("heading", { name: heading, exact: true }));
  await page.reload({ waitUntil: "domcontentloaded" });
  await visible(page.getByRole("heading", { name: heading, exact: true }));
}

export async function liveBrowser(
  config: DashboardCredentials,
  check: LiveCheck,
  machine: components["schemas"]["MachineView"] | undefined,
  organization: components["schemas"]["OrganizationView"] | undefined,
): Promise<void> {
  const browser = await launchBrowser();
  try {
    for (const width of [1440, 720, 390]) {
      await check(
        `live Machines and Settings navigation ${width}`,
        async () => {
          const context = await browser.newContext({
            viewport: { width, height: 1000 },
            locale: "en-GB",
            timezoneId: "UTC",
          });
          try {
            await context.addCookies([
              {
                name: "__Host-ciproxy",
                value: config.session_cookie,
                url: config.base_url,
                secure: true,
                httpOnly: true,
                sameSite: "Lax",
              },
            ]);
            const page = await context.newPage();
            let failed = false;
            page.on("pageerror", () => {
              failed = true;
            });
            const machines = new URL(
              workspaceHref("machines", config.workspace_id),
              config.base_url,
            );
            await page.goto(machines.href, { waitUntil: "domcontentloaded" });
            await reload(page, "Machines");
            if (machine) {
              const detail = new URL(machines);
              detail.searchParams.set("machine", machine.id);
              await page.goto(detail.href, { waitUntil: "domcontentloaded" });
              await reload(page, machine.hostname);
            }
            const settings = new URL(
              workspaceHref("settings", config.workspace_id),
              config.base_url,
            );
            await page.goto(settings.href, { waitUntil: "domcontentloaded" });
            await visible(
              page.getByRole("button", { name: "Add label", exact: true }),
            );
            await reload(page, "Settings");
            await page
              .getByRole("link", { name: "Organizations", exact: true })
              .click();
            await visible(
              page.getByRole("button", { name: "Rename", exact: true }),
            );
            await reload(page, "Settings");
            if (organization) {
              const repositories = new URL(settings);
              repositories.searchParams.set("tab", "organizations");
              repositories.searchParams.set("organization", organization.id);
              await page.goto(repositories.href, {
                waitUntil: "domcontentloaded",
              });
              await visible(
                page.getByRole("radio", {
                  name: "All private repositories",
                  exact: true,
                }),
              );
              await reload(page, "Settings");
            }
            await page.getByRole("link", { name: /^Members/ }).click();
            await visible(
              page.getByRole("heading", { name: "Members", exact: true }),
            );
            await reload(page, "Settings");
            await visible(
              page.getByRole("heading", { name: "Members", exact: true }),
            );
            assert.equal(failed, false);
            // Live pages can show private data. Do not capture screenshots or traces.
          } finally {
            await context.close();
          }
        },
      );
    }
  } finally {
    await browser.close();
  }
}
