import assert from "node:assert/strict";

import type { Page } from "./browser-driver.ts";
import { visible } from "./browser-fixture.ts";
import { workspaceId } from "./fixtures-core.ts";
import { capture, m20Control, machinesUrl } from "./m20-browser-fixture.ts";
import { machineIds, slotJobIds } from "./m20-fixtures-machines.ts";

/**
 * The machine states of the page. Every state saves 2 warm runners, while
 * the slots of a drain report none ready.
 */
const states = [
  [machineIds.offline, "forge-offline"],
  [machineIds.noKvm, "lab-no-kvm"],
  [machineIds.noSlots, "lab-no-slots"],
  [machineIds.diskLimited, "office-mini-1"],
  [machineIds.unknownCapacity, "forge-preparing"],
  [machineIds.draining, "forge-draining"],
] as const;

/** The addresses of the links to the page of one dashboard job. */
async function jobHrefs(page: Page): Promise<string[]> {
  const links = await page.getByRole("link").all();
  const hrefs = await Promise.all(
    links.map((link) => link.getAttribute("href")),
  );
  return hrefs.filter(
    (href): href is string => href !== null && href.startsWith("/job?"),
  );
}

async function stateFacts(page: Page, hostname: string): Promise<void> {
  if (hostname === "forge-draining") {
    await visible(page.getByRole("img", { name: /^1\sbusy\s·\s0\swarm$/ }));
    assert.equal(
      await page.getByText("Warm runner, ready", { exact: true }).count(),
      0,
      "a drain keeps no ready warm runner",
    );
    assert.deepEqual(
      await jobHrefs(page),
      [],
      "a report job that the API did not match links nowhere",
    );
  }
  if (hostname === "forge-offline") {
    await visible(page.getByText("0.0.9", { exact: true }));
    assert.equal(await page.getByText(/up to date/).count(), 0);
  }
  if (hostname === "office-mini-1") {
    await visible(
      page.getByText(/^Disk limits office-mini-1 to 3\sjobs of ciproxy/),
    );
    assert.equal(
      await page.getByText("Free, no disk for a job", { exact: true }).count(),
      3,
    );
  }
}

/**
 * Each machine state shows the saved warm runner count in its control, apart
 * from the ready warm runners of its slots, with the facts of the state.
 */
export async function machineStates(
  page: Page,
  origin: string,
  width: number,
  output: string,
): Promise<void> {
  // A fresh report of an agent one release behind names the newest release.
  await m20Control(origin, "machine-agent-behind");
  await page.goto(machinesUrl(origin, machineIds.primary), {
    waitUntil: "domcontentloaded",
  });
  await visible(page.getByRole("heading", { name: "forge-01", exact: true }));
  await visible(page.getByText(/^0\.4\.1\s·\s0\.4\.2\savailable$/));
  await capture(page, width, output, "forge-01-agent-behind");
  await m20Control(origin, "machines");
  for (const [id, hostname] of states) {
    await page.goto(machinesUrl(origin, id), {
      waitUntil: "domcontentloaded",
    });
    await visible(page.getByRole("heading", { name: hostname, exact: true }));
    assert.equal(
      await page
        .getByRole("status", { name: "Warm runners", exact: true })
        .textContent(),
      "2",
      `${hostname} shows the saved warm runner count`,
    );
    await stateFacts(page, hostname);
    await capture(page, width, output, hostname);
  }
}

/** The busy slots of forge-01 link the canonical dashboard job, once a job. */
export async function slotJobLinks(page: Page): Promise<void> {
  const href = (job: string) =>
    `/job?${new URLSearchParams({ workspace: workspaceId, job }).toString()}`;
  assert.deepEqual(await jobHrefs(page), [
    href(slotJobIds.test),
    href(slotJobIds.build),
    href(slotJobIds.build),
  ]);
  await visible(page.getByRole("link", { name: "test (rust)", exact: true }));
}
