// Readers for the result of a machine command and for the overflow summary
// that the Machines page shows. Each reader returns undefined for a body of
// the wrong shape, so a screen never sees a partial value.

import {
  boolean,
  decimal,
  integer,
  object,
  optional,
  ParseError,
  parser,
  string,
  time,
} from "./parse-core.ts";
import type {
  BoatAccountView,
  BoatSpendView,
  CommandResult,
  OverflowSettings,
} from "./machine-schema.ts";

export const parseCommandResult = parser((value): CommandResult => {
  const fields = object(value, "command result");
  return { connected: boolean(fields.connected) };
});

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

/** The overflow wait, which the contract keeps at zero or more seconds. */
function wait(value: unknown): number {
  const seconds = integer(value);
  if (seconds < 0) throw new ParseError("overflow wait");
  return seconds;
}

export const parseOverflowSummary = parser((value): OverflowSettings => {
  const fields = object(value, "overflow summary");
  return {
    account: optional(fields.account, account),
    available: boolean(fields.available),
    enabled: boolean(fields.enabled),
    next_month_at: time(fields.next_month_at),
    spend: optional(fields.spend, spend),
    spend_limit_cents: optional(fields.spend_limit_cents, decimal),
    wait_seconds: wait(fields.wait_seconds),
  };
});
