import { visible } from "./browser-fixture.ts";
import {
  capture,
  dialogCancel,
  m20Control,
  open,
  selectRole,
  settingsUrl,
  within,
  type M20Case,
} from "./m20-browser-fixture.ts";
import { fixtureMembers } from "./m20-fixtures-settings.ts";
import { labelCases } from "./m20-browser-labels.ts";
import { organizationCases } from "./m20-browser-organizations.ts";

export function settingsCases(origin: string, output: string): M20Case[] {
  return [
    {
      name: "settings-tab-navigation",
      async check(page, _context, width) {
        await open(page, origin, "settings", settingsUrl(origin), "Settings");
        await visible(
          page.getByRole("button", { name: "Add label", exact: true }),
        );
        await capture(page, width, output, "settings-labels");
        await page
          .getByRole("link", { name: "Organizations", exact: true })
          .click();
        await page.waitForURL(settingsUrl(origin, "organizations"));
        await visible(
          page.getByRole("button", { name: "Rename", exact: true }),
        );
        await capture(page, width, output, "settings-organizations");
        await page.getByRole("link", { name: /^Members/ }).click();
        await page.waitForURL(settingsUrl(origin, "members"));
        await visible(
          page.getByRole("button", {
            name: "Give access to rowan",
            exact: true,
          }),
        );
        await page.reload({ waitUntil: "domcontentloaded" });
        await visible(
          page.getByRole("button", {
            name: "Give access to rowan",
            exact: true,
          }),
        );
        await capture(page, width, output, "settings-members");
        await page.goBack({ waitUntil: "domcontentloaded" });
        await visible(
          page.getByRole("button", { name: "Rename", exact: true }),
        );
      },
    },
    {
      name: "member-admin-trust-dialog",
      async check(page, _context, width) {
        await open(
          page,
          origin,
          "settings",
          settingsUrl(origin, "members"),
          "Settings",
        );
        await selectRole(page, "rowan", "admin");
        await page
          .getByRole("button", { name: "Give access to rowan", exact: true })
          .click();
        const title = "Give rowan the Admin role?";
        const dialog = page.getByRole("dialog", { name: title, exact: true });
        await visible(dialog);
        await visible(
          within(dialog).getByText(fixtureMembers().admin_notice, {
            exact: true,
          }),
        );
        await capture(page, width, output, "member-admin-trust-dialog-open");
        await dialogCancel(page, title);
        await selectRole(page, "rowan", "admin");
        await page
          .getByRole("button", { name: "Give access to rowan", exact: true })
          .click();
        await visible(dialog);
        await within(dialog)
          .getByRole("button", { name: "Give the Admin role", exact: true })
          .click();
        await dialog.waitFor({ state: "hidden" });
        await page
          .getByRole("button", { name: "Give access to rowan", exact: true })
          .waitFor({ state: "hidden" });
        await visible(
          page.getByRole("button", { name: "Remove rowan", exact: true }),
        );
      },
    },
    {
      name: "settings-load-recovery",
      expectedHttp: [503],
      async check(page) {
        await open(
          page,
          origin,
          "settings-load-failure",
          settingsUrl(origin),
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
          page.getByRole("button", { name: "Add label", exact: true }),
        );
      },
    },
    {
      name: "members-load-recovery",
      expectedHttp: [503],
      async check(page) {
        await open(
          page,
          origin,
          "members-load-failure",
          settingsUrl(origin, "members"),
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
          page.getByRole("button", {
            name: "Give access to rowan",
            exact: true,
          }),
        );
      },
    },
    ...labelCases(origin, output),
    ...organizationCases(origin, output),
  ];
}
