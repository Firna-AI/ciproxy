// The formats of the Machines pages, after the mockups: money shows cents
// only when it has cents, sizes are whole decimal gigabytes, and a time since
// a contact is short in a table and spelled out in a sentence. Money stays a
// decimal string of cents from the API to the screen, so no amount passes
// through a floating-point number.

/** No-break space: a line never breaks inside a value. */
const nbsp = "\u00a0";

const grouped = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });

/** "4 December" on one line. */
export function unbroken(text: string): string {
  return text.replaceAll(" ", nbsp);
}

/** Values joined by " · ". A line breaks after a dot, never before one. */
export function dotJoin(...parts: readonly string[]): string {
  return parts.join(`${nbsp}· `);
}

/** 1024 -> "1,024". */
export function formatCount(value: number): string {
  return grouped.format(value);
}

/** A byte count in whole decimal gigabytes, without the unit: "1,024". */
export function gigabytes(bytes: string): string {
  return formatCount(Math.round(Number(bytes) / 1e9));
}

/** "5340" -> "$53.40", "4900" -> "$49", "-150" -> "-$1.50". */
export function formatCents(cents: string): string {
  const negative = cents.startsWith("-");
  const digits = (negative ? cents.slice(1) : cents).padStart(3, "0");
  const dollars = digits.slice(0, -2).replace(/^0+(?=\d)/, "");
  const rest = digits.slice(-2);
  const whole = dollars.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  const amount = rest === "00" ? `$${whole}` : `$${whole}.${rest}`;
  return negative ? `-${amount}` : amount;
}

/** "$53.40 a month" on one line. */
export function formatMonthly(cents: string): string {
  return unbroken(`${formatCents(cents)} a month`);
}

/**
 * Dollars that a person typed, as a decimal string of cents, or null when the
 * text is not an amount. A leading "$", grouping commas, and spaces at the
 * ends are allowed; at most two decimals, and never a negative amount.
 */
export function parseDollars(text: string): string | null {
  const plain = text.trim().replace(/^\$\s*/, "");
  if (!/^(\d{1,3}(,\d{3})+|\d+)(\.\d{0,2})?$/.test(plain)) return null;
  const [whole = "", fraction = ""] = plain.replaceAll(",", "").split(".");
  const cents = `${whole}${fraction.padEnd(2, "0")}`.replace(/^0+(?=\d)/, "");
  return cents;
}

/** Cents as the dollars of an edit field: "5340" -> "53.40", "0" -> "0". */
export function centsToInput(cents: string): string {
  const digits = cents.padStart(3, "0");
  const dollars = digits.slice(0, -2).replace(/^0+(?=\d)/, "");
  const rest = digits.slice(-2);
  return rest === "00" ? dollars : `${dollars}.${rest}`;
}

const second = 1_000;
const minute = 60 * second;
const hour = 60 * minute;
const day = 24 * hour;

function elapsed(iso: string, now: number): number {
  return Math.max(0, now - Date.parse(iso));
}

/** "3s ago", "14m ago", "2h ago", "3d ago": a time in a table. */
export function formatAgo(iso: string, now: number): string {
  const span = elapsed(iso, now);
  if (span < minute) return `${Math.floor(span / second)}s ago`;
  if (span < hour) return `${Math.floor(span / minute)}m ago`;
  if (span < day) return `${Math.floor(span / hour)}h ago`;
  return `${Math.floor(span / day)}d ago`;
}

function unit(count: number, one: string): string {
  return unbroken(`${count} ${count === 1 ? one : `${one}s`}`);
}

/** "45 seconds", "14 minutes", "2 hours", "3 days": a time in a sentence. */
export function formatSince(iso: string, now: number): string {
  const span = elapsed(iso, now);
  if (span < minute) return unit(Math.floor(span / second), "second");
  if (span < hour) return unit(Math.floor(span / minute), "minute");
  if (span < day) return unit(Math.floor(span / hour), "hour");
  return unit(Math.floor(span / day), "day");
}

/** The overflow wait as Settings names it: "At once", "1 minute". */
export function waitText(seconds: number): string {
  if (seconds === 0) return "At once";
  if (seconds % 60 !== 0) return unit(seconds, "second");
  return unit(seconds / 60, "minute");
}

/** "20 December": a day in UTC, on one line. */
export function formatDay(iso: string): string {
  const text = new Intl.DateTimeFormat("en-GB", {
    day: "numeric",
    month: "long",
    timeZone: "UTC",
  }).format(new Date(iso));
  return unbroken(text);
}
