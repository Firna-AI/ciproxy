// Number, money, and date formats shared by every screen. One value has one
// format everywhere: money shows cents only when it has cents, except in a
// money column of a table, where every amount shows cents so the column
// aligns. Durations have no space before the unit, such as "6m 12s".

const whole = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });
const cents = new Intl.NumberFormat("en-US", {
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
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

/** Always with cents: 49 -> "$49.00". For a money column of a table. */
export function formatMoneyCents(value: number): string {
  return `$${cents.format(value)}`;
}

/** "4 December" with no-break spaces, so a line never splits the value. */
export function unbroken(text: string): string {
  return text.replaceAll(" ", nbsp);
}

/** "$49 a month", whole on one line. */
export function monthly(amount: number): string {
  return unbroken(`${formatMoney(amount)} a month`);
}

/** Values joined by " · ". A line breaks after a dot, never before one. */
export function dotJoin(...parts: readonly string[]): string {
  return parts.join(`${nbsp}· `);
}

/** "4 November – 3 December": each date whole, the dash with the first. */
export function dateRange(from: string, to: string): string {
  return `${unbroken(from)}${nbsp}– ${unbroken(to)}`;
}
