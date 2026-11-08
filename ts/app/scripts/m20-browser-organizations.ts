import assert from "node:assert/strict";

import { visible } from "./browser-fixture.ts";
import {
  capture,
  dialogCancel,
  fill,
  m20Control,
  open,
  settingsUrl,
  within,
  type M20Case,
} from "./m20-browser-fixture.ts";
import { organizationIds } from "./m20-fixtures-settings.ts";

export function organizationCases(origin: string, output: string): M20Case[] {
  return [
    {
      name: "organization-repository-navigation",
      async check(page, _context, width) {
        await open(
          page,
          origin,
          "settings",
          settingsUrl(origin, "organizations"),
          "Settings",
        );
        await page
          .getByRole("link", {
            name: "Repositories of northstar-labs",
            exact: true,
          })
          .click();
        await page.waitForURL(
          settingsUrl(origin, "organizations", organizationIds.primary),
        );
        await visible(
          page.getByRole("radio", {
            name: "Selected repositories",
            exact: true,
          }),
        );
        const repository = page.getByRole("checkbox", {
          name: "northstar-labs/runner",
          exact: true,
        });
        await repository.click();
        const saved = page.waitForResponse(
          /\/organizations\/[^/]+\/repositories$/,
        );
        await page.getByRole("button", { name: "Save", exact: true }).click();
        await (await saved).finished();
        await page.reload({ waitUntil: "domcontentloaded" });
        await visible(repository);
        assert.equal(await repository.isChecked(), true);
        assert.equal(
          await page
            .getByRole("checkbox", {
              name: "northstar-labs/public-docs",
              exact: true,
            })
            .count(),
          0,
        );
        await capture(page, width, output, "organization-repositories");
        await within(page.getByRole("navigation", { name: "Settings tabs" }))
          .getByRole("link", { name: "Organizations", exact: true })
          .click();
        await page.waitForURL(settingsUrl(origin, "organizations"));
        await visible(
          page.getByRole("button", { name: "Rename", exact: true }),
        );
      },
    },
    {
      name: "organization-take-out-focus",
      async check(page, _context, width) {
        await open(
          page,
          origin,
          "settings",
          settingsUrl(origin, "organizations"),
          "Settings",
        );
        const title = "Take northstar-engineering out of this workspace?";
        await page
          .getByRole("button", {
            name: "Take out northstar-engineering",
            exact: true,
          })
          .click();
        await capture(page, width, output, "organization-take-out-dialog");
        await dialogCancel(page, title);
        // Enter proves that Cancel restored focus to the take-out trigger.
        await page.keyboard.press("Enter");
        const dialog = page.getByRole("dialog", { name: title, exact: true });
        await visible(dialog);
        await within(dialog)
          .getByRole("button", { name: "Take out", exact: true })
          .click();
        await dialog.waitFor({ state: "hidden" });
        await page
          .getByRole("button", {
            name: "Take out northstar-engineering",
            exact: true,
          })
          .waitFor({ state: "hidden" });
        await visible(
          page.getByRole("link", {
            name: "Repositories of northstar-labs",
            exact: true,
          }),
        );
      },
    },
    {
      name: "organization-retention-layout",
      async check(page) {
        await open(
          page,
          origin,
          "organizations-single-with-removed",
          settingsUrl(origin, "organizations"),
          "Settings",
        );
        await visible(page.getByText("northstar-archive", { exact: true }));
        await visible(
          page.getByRole("button", {
            name: "Install the App again on northstar-archive",
            exact: true,
          }),
        );
      },
    },
    {
      name: "repositories-load-recovery",
      expectedHttp: [503],
      async check(page) {
        await open(
          page,
          origin,
          "repositories-load-failure",
          settingsUrl(origin, "organizations", organizationIds.primary),
          "Settings",
        );
        await visible(
          page.getByRole("button", { name: "Try again", exact: true }),
        );
        await m20Control(origin, "settings", false);
        await page
          .getByRole("button", { name: "Try again", exact: true })
          .click();
        await visible(
          page.getByRole("radio", {
            name: "Selected repositories",
            exact: true,
          }),
        );
      },
    },
    {
      name: "organization-both-holds-layout",
      async check(page) {
        await open(
          page,
          origin,
          "organizations-inactive-both",
          settingsUrl(origin, "organizations"),
          "Settings",
        );
        await visible(
          page.getByRole("button", {
            name: "Take out northstar-research",
            exact: true,
          }),
        );
      },
    },
    {
      name: "organization-enable-dialog",
      async check(page, _context, width) {
        await open(
          page,
          origin,
          "organizations-inactive-admin",
          settingsUrl(origin, "organizations"),
          "Settings",
        );
        await within(page.getByLabel("Organizations", { exact: true }))
          .getByRole("button", {
            name: "Turn on northstar-research",
            exact: true,
          })
          .click();
        const dialog = page.getByRole("dialog", {
          name: "Turn on northstar-research?",
          exact: true,
        });
        await visible(dialog);
        await capture(page, width, output, "organization-enable-dialog-open");
        await within(dialog)
          .getByRole("button", { name: "Turn on", exact: true })
          .click();
        await dialog.waitFor({ state: "hidden" });
        await visible(
          page.getByRole("link", {
            name: "Repositories of northstar-research",
            exact: true,
          }),
        );
      },
    },
    {
      name: "workspace-rename-dialog",
      async check(page) {
        await open(
          page,
          origin,
          "settings",
          settingsUrl(origin, "organizations"),
          "Settings",
        );
        await page.getByRole("button", { name: "Rename", exact: true }).click();
        const dialog = page.getByRole("dialog", {
          name: "Rename the workspace",
          exact: true,
        });
        await visible(dialog);
        await fill(page, "Name", "Northstar Operations");
        await within(dialog)
          .getByRole("button", { name: "Save", exact: true })
          .click();
        await dialog.waitFor({ state: "hidden" });
        await visible(
          page.getByRole("button", {
            name: "Workspace Northstar Operations. Switch workspace",
            exact: true,
          }),
        );
      },
    },
  ];
}
