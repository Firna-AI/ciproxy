import assert from "node:assert/strict";

import { visible } from "./browser-fixture.ts";
import {
  addMachine,
  machineControls,
  machineFailure,
} from "./m20-browser-machine-actions.ts";
import {
  capture,
  m20Control,
  machinesUrl,
  open,
  type M20Case,
} from "./m20-browser-fixture.ts";
import { machineIds } from "./m20-fixtures-machines.ts";

export function machineCases(origin: string, output: string): M20Case[] {
  return [
    {
      name: "machines-navigation",
      async check(page) {
        await open(page, origin, "machines", machinesUrl(origin), "Machines");
        await page.getByRole("link", { name: "forge-01", exact: true }).click();
        await page.waitForURL(machinesUrl(origin, machineIds.primary));
        await visible(
          page.getByRole("heading", { name: "forge-01", exact: true }),
        );
        await page.reload({ waitUntil: "domcontentloaded" });
        await visible(
          page.getByRole("heading", { name: "forge-01", exact: true }),
        );
        await page.goBack({ waitUntil: "domcontentloaded" });
        await visible(
          page.getByRole("heading", { name: "Machines", exact: true }),
        );
      },
    },
    {
      name: "machine-health-layout",
      async check(page, _context, width) {
        const states = [
          [machineIds.offline, "forge-offline"],
          [machineIds.noKvm, "lab-no-kvm"],
          [machineIds.noSlots, "lab-no-slots"],
          [machineIds.diskLimited, "office-mini-1"],
          [machineIds.unknownCapacity, "forge-preparing"],
          [machineIds.draining, "forge-draining"],
        ] as const;
        await m20Control(origin, "machines");
        for (const [id, hostname] of states) {
          await page.goto(machinesUrl(origin, id), {
            waitUntil: "domcontentloaded",
          });
          await visible(
            page.getByRole("heading", { name: hostname, exact: true }),
          );
          await capture(page, width, output, hostname);
        }
      },
    },
    {
      name: "machines-long-name",
      async check(page) {
        await open(
          page,
          origin,
          "machines-long-name",
          machinesUrl(origin),
          "Machines",
        );
        await visible(
          page.getByRole("link", {
            name: "northstar-continuous-integration-machine-with-a-long-host-name-01",
            exact: true,
          }),
        );
      },
    },
    {
      name: "machines-empty-add",
      async check(page) {
        await open(
          page,
          origin,
          "machines-empty",
          machinesUrl(origin),
          "Machines",
        );
        await visible(
          page.getByRole("heading", {
            name: "Add your first machine",
            exact: true,
          }),
        );
        await page
          .getByRole("link", { name: "Add machine", exact: true })
          .click();
        await page.waitForURL(machinesUrl(origin, undefined, true));
        await visible(
          page.getByRole("heading", { name: "Add a machine", exact: true }),
        );
      },
    },
    {
      name: "machines-load-recovery",
      expectedHttp: [503],
      async check(page) {
        await open(
          page,
          origin,
          "machines-load-failure",
          machinesUrl(origin),
          "Unable to load machines",
        );
        await m20Control(origin, "machines", false);
        await page
          .getByRole("button", { name: "Try again", exact: true })
          .click();
        await visible(
          page.getByRole("link", { name: "forge-01", exact: true }),
        );
      },
    },
    {
      name: "machine-controls",
      expectedHttp: [503],
      check: (page, _context, width) =>
        machineControls(page, origin, width, output),
    },
    {
      name: "machine-action-failure",
      expectedHttp: [503],
      check: (page) => machineFailure(page, origin),
    },
    {
      name: "machine-removal-navigation",
      async check(page) {
        await open(
          page,
          origin,
          "machines",
          machinesUrl(origin, machineIds.offline),
          "forge-offline",
        );
        await page
          .getByRole("button", { name: "Remove machine", exact: true })
          .click();
        await page.waitForURL(machinesUrl(origin));
        await visible(
          page.getByRole("heading", { name: "Machines", exact: true }),
        );
        await visible(
          page.getByRole("link", { name: "forge-01", exact: true }),
        );
      },
    },
    {
      name: "add-machine-revoked-and-connected",
      expectedHttp: [503],
      check: (page, _context, width) => addMachine(page, origin, width, output),
    },
    {
      name: "install-command-recovery",
      expectedHttp: [503],
      async check(page) {
        await open(
          page,
          origin,
          "install-command-failure",
          machinesUrl(origin, undefined, true),
          "Add a machine",
        );
        await visible(
          page.getByText("Unable to create the install command", {
            exact: true,
          }),
        );
        await m20Control(origin, "add-machine-waiting", false);
        await page
          .getByRole("button", { name: "Try again", exact: true })
          .click();
        await visible(
          page.getByRole("button", { name: "Revoke token", exact: true }),
        );
      },
    },
    {
      name: "machines-viewer",
      async check(page) {
        await open(page, origin, "viewer", machinesUrl(origin), "Machines");
        await visible(
          page.getByRole("link", { name: "forge-01", exact: true }),
        );
        assert.equal(
          await page
            .getByRole("link", { name: "Add machine", exact: true })
            .count(),
          0,
        );
        await page.getByRole("link", { name: "forge-01", exact: true }).click();
        await visible(
          page.getByRole("heading", { name: "forge-01", exact: true }),
        );
        assert.equal(
          await page
            .getByRole("button", { name: "Drain", exact: true })
            .count(),
          0,
        );
      },
    },
  ];
}
