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

const rate = new Intl.NumberFormat("en-US", {
  minimumFractionDigits: 3,
  maximumFractionDigits: 4,
});

/** A price per minute: 0.004 -> "$0.004". */
export function formatRate(value: number): string {
  return `$${rate.format(value)}`;
}

/** "4 December" with no-break spaces, so a line never splits the value. */
export function unbroken(text: string): string {
  return text.replaceAll(" ", nbsp);
}

/** 372 -> "6m 12s", 21 -> "21s": a duration in whole seconds. */
export function formatDuration(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const rest = seconds % 60;
  return minutes === 0
    ? `${rest}s`
    : `${minutes}m ${String(rest).padStart(2, "0")}s`;
}

const tenths = new Intl.NumberFormat("en-US", {
  minimumFractionDigits: 1,
  maximumFractionDigits: 1,
});

/**
 * 1.56 -> "1.6 vCPU": a measured vCPU figure, whole on one line. A VM size or
 * a label size uses vcpuText() of sizes.ts, as the Details panel does.
 */
export function formatVcpu(value: number): string {
  return unbroken(`${tenths.format(value)} vCPU`);
}

/**
 * 6.08 -> "6.1 GB": a measured memory figure in SI GB, whole on one line. A
 * VM size or a label size uses memoryText() of sizes.ts.
 */
export function formatGb(value: number): string {
  return unbroken(`${tenths.format(value)} GB`);
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

const dayMonth = new Intl.DateTimeFormat("en-GB", {
  day: "numeric",
  month: "long",
  timeZone: "UTC",
});

/** A day of the story in UTC, such as "20 December", whole on one line. */
export function formatDay(date: Date): string {
  return unbroken(dayMonth.format(date));
}
