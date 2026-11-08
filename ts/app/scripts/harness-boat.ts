// The Boat scenarios of the fixture API. A workspace that never connected
// Boat is the default, so every page of every scenario can read the Boat
// settings. The Boat routes live in harness-boat-api.ts.
import type { OverflowSettings } from "../src/api/schema.ts";
import { boatSpend, notConnected, overflowSettings } from "./fixtures-boat.ts";
import { job, machine } from "./fixtures-setup.ts";
import type { Scenario } from "./harness-scenarios.ts";

export const boatScenarios = [
  "boat-setup",
  "boat-setup-rejected",
  "boat-setup-unavailable",
  "boat-setup-load-failure",
  "boat-setup-job-running",
  "overflow-connected",
  "overflow-not-connected",
  "overflow-rejected",
  "overflow-limit-reached",
  "overflow-unknown-costs",
  "overflow-spend-unavailable",
  "overflow-unavailable",
  "overflow-load-failure",
  "overflow-save-failure",
  "overflow-connect-failure",
  "overflow-refresh-after-connect",
  "overflow-change-key-held",
] as const;

export type BoatFixture = {
  settings: OverflowSettings;
  /** Every read of the settings fails. */
  readFails: boolean;
  /** The read after each accepted change fails once. */
  failReadAfterChange: boolean;
  nextReadFails: boolean;
  connectFails: boolean;
  saveFails: boolean;
  /** A connection waits until the browser check releases it. */
  holdConnection: boolean;
  release: (() => void) | null;
  /** The field names of the last connection body, never its values. */
  connectionFields: string[] | null;
};

export function defaultBoat(): BoatFixture {
  return {
    settings: notConnected(),
    readFails: false,
    failReadAfterChange: false,
    nextReadFails: false,
    connectFails: false,
    saveFails: false,
    holdConnection: false,
    release: null,
    connectionFields: null,
  };
}

export type BoatScenario = (typeof boatScenarios)[number];

function isBoatScenario(name: string): name is BoatScenario {
  return (boatScenarios as readonly string[]).includes(name);
}

function rejectedAccount(): OverflowSettings {
  const settings = overflowSettings();
  const account = settings.account ?? null;
  return {
    ...settings,
    account: account && { ...account, rejected_at: "2026-11-12T08:30:00Z" },
  };
}

function settingsFor(name: BoatScenario): OverflowSettings {
  switch (name) {
    case "boat-setup-rejected":
    case "overflow-rejected":
      return rejectedAccount();
    case "boat-setup-unavailable":
    case "overflow-unavailable":
      return { ...notConnected(), available: false };
    case "boat-setup":
    case "boat-setup-load-failure":
    case "boat-setup-job-running":
    case "overflow-not-connected":
    case "overflow-connect-failure":
    case "overflow-refresh-after-connect":
      return notConnected();
    case "overflow-limit-reached":
      return overflowSettings({
        spend: boatSpend({
          confirmed_cents: "18640",
          unconfirmed_cents: "1360",
          limit_reached: true,
        }),
      });
    case "overflow-unknown-costs":
      return overflowSettings({
        spend: boatSpend({ bounded: false, limit_reached: null }),
      });
    case "overflow-spend-unavailable":
      return overflowSettings({ spend: null });
    case "overflow-connected":
    case "overflow-load-failure":
    case "overflow-save-failure":
    case "overflow-change-key-held":
      return overflowSettings();
  }
}

export function applyBoatScenario(data: Scenario): void {
  const name: string = data.name;
  if (!isBoatScenario(name)) return;
  const boat = data.boat;
  boat.settings = settingsFor(name);
  boat.readFails = name.endsWith("-load-failure");
  boat.connectFails = name === "overflow-connect-failure";
  boat.saveFails = name === "overflow-save-failure";
  boat.failReadAfterChange = name === "overflow-refresh-after-connect";
  boat.holdConnection = name === "overflow-change-key-held";
  if (name.startsWith("boat-setup")) {
    data.machines = [machine()];
    data.setup.machines = 1;
    data.setup.waiting_for_machine = false;
  }
  if (name === "boat-setup-job-running") data.setup.first_job = job();
}
