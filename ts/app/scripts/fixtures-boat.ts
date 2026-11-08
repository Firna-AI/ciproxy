// Recorded Boat settings of the fixture workspace. Every body uses the
// generated Rust contract. The accepted key is a fixture word that the
// harness accepts; it is not a Boat key and opens nothing.
import type { BoatSpendView, OverflowSettings } from "../src/api/schema.ts";

/** The one key that the fixture API accepts. */
export const fixtureBoatKey = "fixture-only-accepted-key";

/** A key that the fixture API refuses, as Boat refuses a wrong key. */
export const refusedBoatKey = "fixture-only-refused-key";

export function boatSpend(
  overrides: Partial<BoatSpendView> = {},
): BoatSpendView {
  return {
    bounded: true,
    confirmed_cents: "4810",
    limit_reached: false,
    settled_bound_cents: "120",
    unconfirmed_cents: "1310",
    ...overrides,
  } satisfies BoatSpendView;
}

export function overflowSettings(
  overrides: Partial<OverflowSettings> = {},
): OverflowSettings {
  return {
    account: {
      connected_at: "2026-11-02T09:00:00Z",
      name: "northstar",
      rejected_at: null,
    },
    available: true,
    enabled: true,
    next_month_at: "2026-12-01T00:00:00Z",
    spend: boatSpend(),
    spend_limit_cents: "20000",
    wait_seconds: 30,
    ...overrides,
  } satisfies OverflowSettings;
}

/** A workspace that never connected Boat. */
export function notConnected(): OverflowSettings {
  return overflowSettings({
    account: null,
    spend: null,
    spend_limit_cents: null,
  });
}
