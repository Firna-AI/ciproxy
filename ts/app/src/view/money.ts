// US-dollar amounts. The API sends integer cents as decimal strings, and a
// person types a limit in dollars. Both directions work on whole numbers, so
// no amount passes through a binary fraction and no cent is lost.

const grouped = new Intl.NumberFormat("en-US");

function parts(cents: string): { sign: string; dollars: bigint; rest: bigint } {
  const value = BigInt(cents);
  const absolute = value < 0n ? -value : value;
  return {
    sign: value < 0n ? "-" : "",
    dollars: absolute / 100n,
    rest: absolute % 100n,
  };
}

function centsText(rest: bigint): string {
  return rest.toString().padStart(2, "0");
}

/** "$200" for whole dollars, else "$61.20", with commas for thousands. */
export function formatCents(cents: string): string {
  const { sign, dollars, rest } = parts(cents);
  const whole = `${sign}$${grouped.format(dollars)}`;
  return rest === 0n ? whole : `${whole}.${centsText(rest)}`;
}

/** The text of a limit field: "200", or "200.50" with cents. */
export function dollarsText(cents: string): string {
  const { sign, dollars, rest } = parts(cents);
  const whole = `${sign}${dollars.toString()}`;
  return rest === 0n ? whole : `${whole}.${centsText(rest)}`;
}

export type DollarsInput = { ok: true; cents: string } | { ok: false };

const amount =
  /^\$?\s*(?:(\d{1,3}(?:,\d{3})+|\d+)(?:\.(\d{0,2}))?|\.(\d{1,2}))$/;

/**
 * A typed amount above zero in US dollars, such as "200", "1,000", "$49.5",
 * "49.50", or ".50". More than two decimals or no digit is not an amount.
 */
export function parseDollars(text: string): DollarsInput {
  const match = amount.exec(text.trim());
  if (match === null) return { ok: false };
  const dollars = BigInt((match[1] ?? "0").replaceAll(",", ""));
  const rest = BigInt((match[2] ?? match[3] ?? "").padEnd(2, "0"));
  const cents = dollars * 100n + rest;
  return cents > 0n ? { ok: true, cents: cents.toString() } : { ok: false };
}

/** The share of `limit` that `part` covers, from 0 to 1. */
export function shareOf(part: string, limit: string): number {
  const whole = BigInt(limit);
  if (whole <= 0n) return 0;
  const share = Number((BigInt(part) * 10_000n) / whole) / 10_000;
  return Math.min(Math.max(share, 0), 1);
}
