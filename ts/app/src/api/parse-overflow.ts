// Credential-free provider settings, connection results, and Boat wallets.

import {
  array,
  boolean,
  decimal,
  integer,
  object,
  oneOf,
  optional,
  parser,
  string,
  time,
} from "./parse-core.ts";
import type {
  BoatWallet,
  BoatWallets,
  OverflowProvider,
  OverflowSettings,
  ProviderAccountView,
  ProviderSettings,
  ProviderSpendView,
  ProviderSuccess,
} from "./schema.ts";

export const readProvider = oneOf<OverflowProvider>({ boat: true });

function account(value: unknown): ProviderAccountView {
  const fields = object(value, "provider account");
  return {
    connected_at: time(fields.connected_at),
    name: string(fields.name),
    rejected_at: optional(fields.rejected_at, time),
  };
}

function spend(value: unknown): ProviderSpendView {
  const fields = object(value, "provider spend");
  return {
    bounded: boolean(fields.bounded),
    confirmed_cents: decimal(fields.confirmed_cents),
    limit_reached: optional(fields.limit_reached, boolean),
    settled_bound_cents: decimal(fields.settled_bound_cents),
    unconfirmed_cents: decimal(fields.unconfirmed_cents),
  };
}

function providerSettings(value: unknown): ProviderSettings {
  const fields = object(value, "provider settings");
  return {
    account: optional(fields.account, account),
    available: boolean(fields.available),
    has_billing_account_choice: boolean(fields.has_billing_account_choice),
    name: string(fields.name),
    offered: boolean(fields.offered),
    provider: readProvider(fields.provider),
    spend: optional(fields.spend, spend),
    spend_limit_cents: optional(fields.spend_limit_cents, decimal),
  };
}

export function readOverflowSettings(value: unknown): OverflowSettings {
  const fields = object(value, "overflow settings");
  return {
    enabled: boolean(fields.enabled),
    next_month_at: time(fields.next_month_at),
    providers: array(fields.providers, providerSettings),
    wait_seconds: integer(fields.wait_seconds),
  };
}

export const parseOverflowSettings = parser(readOverflowSettings);

export const parseProviderSuccess = parser((value): ProviderSuccess => {
  const fields = object(value, "provider result");
  return { ok: boolean(fields.ok), provider: readProvider(fields.provider) };
});

function wallet(value: unknown): BoatWallet {
  const fields = object(value, "Boat wallet");
  return { name: string(fields.name), wallet: string(fields.wallet) };
}

export const parseBoatWallets = parser((value): BoatWallets => {
  const fields = object(value, "Boat wallets");
  return {
    provider: readProvider(fields.provider),
    selected_wallet: string(fields.selected_wallet),
    wallets: array(fields.wallets, wallet),
  };
});
