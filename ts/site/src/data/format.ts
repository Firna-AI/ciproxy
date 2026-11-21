// The number, money, and text formats of the site, after the formats of the
// mockups. One value has one format everywhere: money shows cents only when
// it has cents, except in a money column, where every amount shows cents so
// that the column aligns.

const whole = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });

const cents = new Intl.NumberFormat("en-US", {
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});

const rate = new Intl.NumberFormat("en-US", {
  minimumFractionDigits: 3,
  maximumFractionDigits: 4,
});

/** No-break space: a line break never falls inside the value. */
const nbsp = "\u00a0";

/** 17772 -> "17,772". */
export function formatCount(value: number): string {
  return whole.format(value);
}

/** 49 -> "$49", 61.2 -> "$61.20". */
export function formatMoney(value: number): string {
  return Number.isInteger(value)
    ? `$${whole.format(value)}`
    : `$${cents.format(value)}`;
}

/** Always with cents: 49 -> "$49.00". For a money column. */
export function formatMoneyCents(value: number): string {
  return `$${cents.format(value)}`;
}

/** A price per minute: 0.004 -> "$0.004". */
export function formatRate(value: number): string {
  return `$${rate.format(value)}`;
}

/** "4 November 2026" with no-break spaces, so a line never splits it. */
export function unbroken(text: string): string {
  return text.replaceAll(" ", nbsp);
}

/** "a, b, and c": the parts of a sentence, with "and" before the last. */
export function listText(parts: readonly string[]): string {
  if (parts.length < 3) return parts.join(" and ");
  return `${parts.slice(0, -1).join(", ")}, and ${parts.at(-1) ?? ""}`;
}
