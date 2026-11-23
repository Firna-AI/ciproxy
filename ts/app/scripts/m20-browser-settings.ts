import assert from "node:assert/strict";

import { siteLinks } from "../src/view/page-model.ts";
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
import {
  labelFacts,
  memberFacts,
  organizationFacts,
  overflowTabFacts,
} from "./m20-browser-settings-facts.ts";
import { labelOrderCases } from "./m20-browser-label-order.ts";
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
        await labelFacts(page, width);
        await capture(page, width, output, "settings-labels");
        await page
          .getByRole("link", { name: "Organizations", exact: true })
          .click();
        await page.waitForURL(settingsUrl(origin, "organizations"));
        await visible(
          page.getByRole("button", { name: "Rename", exact: true }),
        );
        await organizationFacts(page);
        await capture(page, width, output, "settings-organizations");
        await page.getByRole("link", { name: /^Members/ }).click();
        await page.waitForURL(settingsUrl(origin, "members"));
        await visible(
          page.getByRole("button", {
            name: "Give access to Rowan Hale",
            exact: true,
          }),
        );
        await page.reload({ waitUntil: "domcontentloaded" });
        await visible(
          page.getByRole("button", {
            name: "Give access to Rowan Hale",
            exact: true,
          }),
        );
        await memberFacts(page, width);
        await capture(page, width, output, "settings-members");
        await page.goBack({ waitUntil: "domcontentloaded" });
        await visible(
          page.getByRole("button", { name: "Rename", exact: true }),
        );
        await overflowTabFacts(page, origin);
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
        await selectRole(page, "Rowan Hale", "admin");
        await page
          .getByRole("button", {
            name: "Give access to Rowan Hale",
            exact: true,
          })
          .click();
        const title = "Give Rowan Hale the Admin role?";
        const dialog = page.getByRole("dialog", { name: title, exact: true });
        await visible(dialog);
        await visible(
          within(dialog).getByText(fixtureMembers().admin_notice, {
            exact: true,
          }),
        );
        const termsLine = within(dialog).getByText(
          "An Admin also accepts new versions of the ciproxy Terms for the customer that holds this workspace.",
          { exact: true },
        );
        await visible(termsLine);
        const terms = termsLine.getByRole("link", {
          name: "Terms",
          exact: true,
        });
        assert.equal(await terms.getAttribute("href"), siteLinks.terms);
        assert.equal(await terms.getAttribute("target"), "_blank");
        // The dialog opens on Cancel, not on the Terms link before it, so Enter
        // cannot open the Terms by mistake. No locator gives the focus, so the
        // page reads it, once: the frame moves the focus as the dialog appears.
        // A phone sheet then loses that focus for another reason, so the check
        // asserts only that the Terms link does not hold it.
        assert.notEqual(
          await page.evaluate(
            () => document.activeElement?.getAttribute("href") ?? null,
            undefined,
          ),
          siteLinks.terms,
          "The dialog opened with the focus on its Terms link",
        );
        await capture(page, width, output, "member-admin-trust-dialog-open");
        await dialogCancel(page, title);
        await selectRole(page, "Rowan Hale", "admin");
        await page
          .getByRole("button", {
            name: "Give access to Rowan Hale",
            exact: true,
          })
          .click();
        await visible(dialog);
        await within(dialog)
          .getByRole("button", { name: "Give the Admin role", exact: true })
          .click();
        await dialog.waitFor({ state: "hidden" });
        await page
          .getByRole("button", {
            name: "Give access to Rowan Hale",
            exact: true,
          })
          .waitFor({ state: "hidden" });
        await visible(
          page.getByRole("button", { name: "Remove Rowan Hale", exact: true }),
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
            name: "Give access to Rowan Hale",
            exact: true,
          }),
        );
      },
    },
    ...labelCases(origin, output),
    ...labelOrderCases(origin, output),
    ...organizationCases(origin, output),
  ];
}
