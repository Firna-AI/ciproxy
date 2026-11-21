import assert from "node:assert/strict";

import { workspaceHomeHref } from "../src/routing/pages.ts";
import type { SmokeCase } from "./browser-cases.ts";
import type { Locator, Page } from "./browser-driver.ts";
import { insideViewport, visible } from "./browser-fixture.ts";
import {
  longRenamedWorkspaceName,
  longWorkspaceName,
  workspaceId,
} from "./fixtures-core.ts";

async function fullLabelInsideControl(
  page: Page,
  role: "button" | "link",
  name: string,
  width: number,
): Promise<Locator> {
  const control = page.getByRole(role, { name, exact: true });
  const label = page.getByText(name, { exact: true });
  await insideViewport(control, width);
  await visible(label);
  assert.equal(await label.textContent(), name, "Keep the full action label");
  const controlBox = await control.boundingBox();
  const labelBox = await label.boundingBox();
  assert.ok(controlBox !== null, `The ${role} has no visible bounds`);
  assert.ok(labelBox !== null, `The label has no visible bounds: ${name}`);
  assert.ok(
    labelBox.x >= controlBox.x - 1 &&
      labelBox.y >= controlBox.y - 1 &&
      labelBox.x + labelBox.width <= controlBox.x + controlBox.width + 1 &&
      labelBox.y + labelBox.height <= controlBox.y + controlBox.height + 1,
    `The full label exceeds its ${role}: ${name}. ${JSON.stringify({ controlBox, labelBox })}`,
  );
  return control;
}

export async function exerciseLongActionCase(
  page: Page,
  item: SmokeCase,
  width: number,
): Promise<void> {
  if (item.name === "links-long-names-visible") {
    const choice = page.getByRole("radio", {
      name: new RegExp(`^${longWorkspaceName}`),
    });
    await visible(choice);
    assert.equal(await choice.isChecked(), false);
    const unpicked = page.getByRole("button", {
      name: `Add ${longWorkspaceName}`,
      exact: true,
    });
    assert.equal(await unpicked.isEnabled(), false);
    await choice.click();
    assert.equal(await choice.isChecked(), true);
    const add = await fullLabelInsideControl(
      page,
      "button",
      `Add ${longWorkspaceName} to ${longWorkspaceName}`,
      width,
    );
    assert.equal(await add.isEnabled(), true);
    await add.click();
    await visible(
      page.getByRole("heading", {
        name: `${longWorkspaceName} joined ${longWorkspaceName}, inactive`,
        exact: true,
      }),
    );
  }
  if (
    item.name === "links-long-names-visible" ||
    item.name === "links-renamed-open-visible"
  ) {
    const name =
      item.name === "links-long-names-visible"
        ? longWorkspaceName
        : longRenamedWorkspaceName;
    const open = await fullLabelInsideControl(
      page,
      "link",
      `Open ${name}`,
      width,
    );
    assert.equal(
      await open.getAttribute("href"),
      workspaceHomeHref(workspaceId),
    );
  }
}
