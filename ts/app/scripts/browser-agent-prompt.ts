import assert from "node:assert/strict";

import { agentPrompt } from "../src/view/setup-agent-prompt.ts";
import type { Page } from "./browser-driver.ts";
import { control, visible } from "./browser-fixture.ts";
import { workspaceId } from "./fixtures-core.ts";
import { machineFitLabels } from "./fixtures-setup.ts";

/** The first sentence of the prompt for a coding agent. */
const promptStart =
  "Move the GitHub Actions jobs of this repository to ciproxy runners.";

/**
 * With a denied clipboard, Copy prompt for your agent shows the prompt, so
 * that the person can select it and copy it. Before the press, the prompt
 * stays off the page.
 */
export async function agentPromptCopyFailure(
  page: Page,
  origin: string,
): Promise<void> {
  await control(origin, { scenario: "setup-available" });
  await page.goto(`${origin}/setup?workspace=${workspaceId}`, {
    waitUntil: "domcontentloaded",
  });
  const button = page.getByRole("button", {
    name: "Copy prompt for your agent",
    exact: true,
  });
  await visible(button);
  const prompt = page.getByText(promptStart, { exact: false });
  assert.equal(await prompt.count(), 0, "The prompt stays off the page");
  await button.click();
  await visible(
    page.getByText("Unable to copy the prompt for your agent", {
      exact: true,
    }),
  );
  await visible(prompt);
}

/**
 * With clipboard access, Copy prompt for your agent writes the whole prompt
 * for the labels of the `setup-available` scenario, and the button then says
 * that it copied the prompt. The person pastes this text into an agent.
 */
export async function agentPromptCopy(page: Page): Promise<void> {
  await page
    .getByRole("button", { name: "Copy prompt for your agent", exact: true })
    .click();
  await visible(
    page.getByRole("button", {
      name: "Copied the prompt for your agent",
      exact: true,
    }),
  );
  const pasted = await page.evaluate(
    () => navigator.clipboard.readText(),
    undefined,
  );
  assert.equal(pasted, agentPrompt(machineFitLabels()));
}
