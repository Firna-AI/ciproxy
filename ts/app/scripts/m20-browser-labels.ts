import assert from "node:assert/strict";

import type { Page } from "./browser-driver.ts";
import { visible } from "./browser-fixture.ts";
import {
  capture,
  dialogCancel,
  fill,
  open,
  settingsUrl,
  within,
  type M20Case,
} from "./m20-browser-fixture.ts";

async function addLabelDialog(
  page: Page,
  origin: string,
  scenario:
    "settings" | "label-refused" | "label-no-fit" | "settings-mutation-failure",
) {
  await open(page, origin, scenario, settingsUrl(origin), "Settings");
  await page.getByRole("button", { name: "Add label", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "Add a label", exact: true });
  await visible(dialog);
  return within(dialog);
}

export function labelCases(origin: string, output: string): M20Case[] {
  return [
    {
      name: "label-dialog-focus-and-save",
      async check(page, _context, width) {
        await addLabelDialog(page, origin, "settings");
        await capture(page, width, output, "label-fit-range");
        await dialogCancel(page, "Add a label");
        // Enter proves that the modal restored focus to Add label.
        await page.keyboard.press("Enter");
        const dialog = page.getByRole("dialog", {
          name: "Add a label",
          exact: true,
        });
        await visible(dialog);
        await fill(page, "Label", "browser-smoke");
        await fill(page, "vCPU", "2");
        await fill(page, "Memory, GB", "8");
        await fill(page, "Free disk, GB", "1");
        await within(dialog)
          .getByRole("button", { name: "Add label", exact: true })
          .click();
        await dialog.waitFor({ state: "hidden" });
        await visible(
          page.getByRole("button", { name: "Edit browser-smoke", exact: true }),
        );
        await page.reload({ waitUntil: "domcontentloaded" });
        await page
          .getByRole("button", { name: "Edit browser-smoke", exact: true })
          .click();
        const edit = page.getByRole("dialog", {
          name: "Edit browser-smoke",
          exact: true,
        });
        await visible(edit);
        await capture(page, width, output, "label-edit-dialog");
        await within(edit)
          .getByRole("button", { name: "Delete label", exact: true })
          .click();
        await edit.waitFor({ state: "hidden" });
        await page
          .getByRole("button", { name: "Edit browser-smoke", exact: true })
          .waitFor({ state: "hidden" });
      },
    },
    {
      name: "label-github-refusal",
      expectedHttp: [422],
      async check(page) {
        const dialog = await addLabelDialog(page, origin, "label-refused");
        await fill(page, "Label", "github-refused-fixture");
        await dialog
          .getByRole("button", { name: "Add label", exact: true })
          .click();
        await visible(
          page.getByText(
            "GitHub refused this label name. Choose another name.",
            { exact: true },
          ),
        );
        await visible(
          page.getByRole("textbox", { name: "Label", exact: true }),
        );
      },
    },
    {
      name: "label-no-machine-fit",
      async check(page, _context, width) {
        const dialog = await addLabelDialog(page, origin, "label-no-fit");
        await visible(dialog.getByRole("radio", { name: "Wait", exact: true }));
        await capture(page, width, output, "label-no-fit-dialog");
      },
    },
    {
      name: "label-partial-fit-layout",
      async check(page, _context, width) {
        await open(
          page,
          origin,
          "label-partial-fit",
          settingsUrl(origin),
          "Settings",
        );
        await page
          .getByRole("button", { name: "Edit ciproxy-8vcpu", exact: true })
          .click();
        await visible(
          page.getByRole("dialog", { name: "Edit ciproxy-8vcpu", exact: true }),
        );
        await capture(page, width, output, "label-partial-fit-dialog");
      },
    },
    {
      name: "label-save-failure",
      expectedHttp: [503],
      async check(page) {
        const dialog = await addLabelDialog(
          page,
          origin,
          "settings-mutation-failure",
        );
        await fill(page, "Label", "save-failure-fixture");
        await dialog
          .getByRole("button", { name: "Add label", exact: true })
          .click();
        await visible(
          page.getByText("Unable to save the label", { exact: true }),
        );
        assert.equal(
          await dialog
            .getByRole("button", { name: "Cancel", exact: true })
            .count(),
          1,
        );
      },
    },
  ];
}
