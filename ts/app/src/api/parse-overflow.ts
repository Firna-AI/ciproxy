// Readers for the Boat settings of Setup and Settings under Overflow: the
// account, the overflow switch and wait, the saved spend limit, and the
// spend of the month. The account never holds the key.

import {
  boolean,
  decimal,
  integer,
  object,
  optional,
  parser,
  string,
  time,
} from "./parse-core.ts";
import type {
  BoatAccountView,
  BoatSpendView,
  OverflowSettings,
} from "./schema.ts";

function account(value: unknown): BoatAccountView {
  const fields = object(value, "Boat account");
  return {
    connected_at: time(fields.connected_at),
    name: string(fields.name),
    rejected_at: optional(fields.rejected_at, time),
  };
}

function spend(value: unknown): BoatSpendView {
  const fields = object(value, "Boat spend");
  return {
    bounded: boolean(fields.bounded),
    confirmed_cents: decimal(fields.confirmed_cents),
    limit_reached: optional(fields.limit_reached, boolean),
    settled_bound_cents: decimal(fields.settled_bound_cents),
    unconfirmed_cents: decimal(fields.unconfirmed_cents),
  };
}

export const parseOverflowSettings = parser((value): OverflowSettings => {
  const fields = object(value, "overflow settings");
  return {
    account: optional(fields.account, account),
    available: boolean(fields.available),
    enabled: boolean(fields.enabled),
    next_month_at: time(fields.next_month_at),
    spend: optional(fields.spend, spend),
    spend_limit_cents: optional(fields.spend_limit_cents, decimal),
    wait_seconds: integer(fields.wait_seconds),
  };
});
