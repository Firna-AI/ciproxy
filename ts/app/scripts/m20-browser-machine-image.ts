// A machine that gets its first job image on the Machines pages, in the
// browser: its row and the figures of the list, its page with the notice and
// the slots, and Add a machine when it joins. forge-preparing of the fixtures
// has no selected image yet, so the API gives its state as preparing.

import type { Page } from "./browser-driver.ts";
import { visible } from "./browser-fixture.ts";
import {
  capture,
  m20Control,
  machinesUrl,
  open,
  phone,
  within,
} from "./m20-browser-fixture.ts";
import { machineIds } from "./m20-fixtures-machines.ts";

/**
 * The list shows Preparing and the image wait in place of a slot bar, and
 * Slots in use counts only the 18 slots of forge-01, office-mini-1, and
 * forge-draining. A phone shows cards, so it checks the page as a whole.
 */
async function list(
  page: Page,
  origin: string,
  width: number,
  output: string,
): Promise<void> {
  await open(page, origin, "machines", machinesUrl(origin), "Machines");
  const slots = page.getByRole("group", { name: "Slots in use", exact: true });
  await visible(within(slots).getByText("4 of 18", { exact: true }));
  const row = phone(width)
    ? page
    : within(page.getByRole("row", { name: /^forge-preparing\s/ }));
  await visible(row.getByText("Preparing", { exact: true }));
  const caption = row.getByText("Getting the job image", { exact: true });
  await visible(caption);
  await caption.scrollIntoViewIfNeeded();
  await capture(page, width, output, "machines-list-preparing");
}

/** The page says when jobs start on the machine, and where they run until then. */
async function machinePage(
  page: Page,
  origin: string,
  width: number,
  output: string,
): Promise<void> {
  await page
    .getByRole("link", { name: "forge-preparing", exact: true })
    .click();
  await page.waitForURL(machinesUrl(origin, machineIds.unknownCapacity));
  await visible(
    page.getByRole("heading", { name: "forge-preparing", exact: true }),
  );
  await visible(
    page.getByText("forge-preparing is getting the job image", {
      exact: true,
    }),
  );
  await visible(
    page.getByText(
      "Jobs start on it when it is ready. Until then, jobs run on your other machines and on Boat.",
      { exact: true },
    ),
  );
  await visible(
    page.getByText("Slots take jobs when the machine is ready.", {
      exact: true,
    }),
  );
  await visible(
    page.getByText(
      /^6\sslots of 2\svCPU and 8\sGB\. A larger VM size takes several slots\.$/,
    ),
  );
  await capture(page, width, output, "machine-page-preparing");
}

/** A machine that joins and gets its first job image says so under its facts. */
async function joins(
  page: Page,
  origin: string,
  width: number,
  output: string,
): Promise<void> {
  await open(
    page,
    origin,
    "add-machine-waiting",
    machinesUrl(origin, undefined, true),
    "Add a machine",
  );
  await visible(
    page.getByText("Waiting for the machine to connect…", { exact: true }),
  );
  await m20Control(origin, "add-machine-preparing", false);
  await page.clock.runFor(5001);
  await visible(page.getByText("forge-preparing connected", { exact: true }));
  await visible(
    page.getByText(
      "Getting the job image. Jobs start on it when it is ready.",
      { exact: true },
    ),
  );
  await capture(page, width, output, "add-machine-preparing");
}

export async function machineImage(
  page: Page,
  origin: string,
  width: number,
  output: string,
): Promise<void> {
  await list(page, origin, width, output);
  await machinePage(page, origin, width, output);
  await joins(page, origin, width, output);
}
