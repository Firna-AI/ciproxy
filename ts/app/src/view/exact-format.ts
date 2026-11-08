// Exact formats of API values: counts, minutes, and US cents as decimal
// strings, and UTC dates as YYYY-MM-DD. Counts and cents never pass through a
// float, so a large value keeps every digit. A money amount shows cents only
// when it has cents, except in a money column, where every amount shows them
// so the column aligns.

/** No-break space: a line never breaks inside the value. */
const nbsp = "\u00a0";

/** The minus sign, which screen readers announce as "minus". */
const minus = "\u2212";

/** "4 December" with no-break spaces, so a line never splits the value. */
export function unbroken(value: string): string {
  return value.replaceAll(" ", nbsp);
}

/** Values joined by " · ". A line breaks after a dot, never before one. */
export function dotJoin(...parts: readonly string[]): string {
  return parts.join(`${nbsp}· `);
}

/** Split a decimal string into its sign, whole digits, and fraction digits. */
function parts(value: string): {
  negative: boolean;
  whole: string;
  fraction: string;
} {
  const match = /^(-?)(\d+)(?:\.(\d+))?$/.exec(value.trim());
  if (match === null) throw new Error(`Not a decimal string: ${value}`);
  return {
    negative: match[1] === "-",
    whole: match[2] ?? "0",
    fraction: match[3] ?? "",
  };
}

/** "17760" -> "17,760". */
function grouped(digits: string): string {
  const plain = digits.replace(/^0+(?=\d)/, "");
  return plain.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

/** A whole count in decimal-string form: "17760" -> "17,760". */
export function formatCount(value: string | bigint): string {
  const { negative, whole } = parts(String(value));
  const text = grouped(whole);
  return negative && text !== "0" ? `${minus}${text}` : text;
}

/**
 * An exact decimal rounded to a whole number, half away from zero:
 * "80604.5" -> "80,605". For values such as fractional minutes.
 */
export function formatWhole(value: string): string {
  const { negative, whole, fraction } = parts(value);
  let rounded = BigInt(whole);
  if ((fraction[0] ?? "0") >= "5") rounded += 1n;
  const text = grouped(rounded.toString());
  return negative && text !== "0" ? `${minus}${text}` : text;
}

/** The sum of exact decimal strings, kept exact: "1.5" + "2.25" -> "3.75". */
export function sumDecimals(values: readonly string[]): string {
  const read = values.map(parts);
  const scale = Math.max(0, ...read.map((value) => value.fraction.length));
  const total = read.reduce((sum, value) => {
    const digits = BigInt(value.whole + value.fraction.padEnd(scale, "0"));
    return sum + (value.negative ? -digits : digits);
  }, 0n);
  const negative = total < 0n;
  const digits = (negative ? -total : total)
    .toString()
    .padStart(scale + 1, "0");
  const whole = digits.slice(0, digits.length - scale);
  const fraction = digits.slice(digits.length - scale).replace(/0+$/, "");
  const text = fraction === "" ? whole : `${whole}.${fraction}`;
  return negative ? `-${text}` : text;
}

/** -1, 0, or 1: the sign of an amount of cents. */
export function centsSign(cents: string): -1 | 0 | 1 {
  const { negative, whole } = parts(cents);
  if (/^0+$/.test(whole)) return 0;
  return negative ? -1 : 1;
}

/** The amount without its sign: "-12345" -> "12345". */
export function absoluteCents(cents: string): string {
  return parts(cents).whole;
}

export type MoneyStyle = "short" | "column";

/**
 * US cents as dollars: "4900" -> "$49", "6120" -> "$61.20". The column style
 * always shows cents: "4900" -> "$49.00".
 */
export function formatCents(
  cents: string,
  style: MoneyStyle = "short",
): string {
  const { negative, whole } = parts(cents);
  const padded = whole.padStart(3, "0");
  const dollars = grouped(padded.slice(0, -2));
  const rest = padded.slice(-2);
  const amount =
    style === "short" && rest === "00" ? `$${dollars}` : `$${dollars}.${rest}`;
  return negative && centsSign(cents) !== 0 ? `${minus}${amount}` : amount;
}

const utcDay = /^(\d{4})-(\d{2})-(\d{2})$/;

function dayTime(day: string): number {
  const match = utcDay.exec(day);
  if (match === null) throw new Error(`Not a UTC date: ${day}`);
  return Date.UTC(Number(match[1]), Number(match[2]) - 1, Number(match[3]));
}

/** A UTC date moved by whole days: ("2026-11-07", -1) -> "2026-11-06". */
export function addUtcDays(day: string, days: number): string {
  return new Date(dayTime(day) + days * 86_400_000).toISOString().slice(0, 10);
}

/** The UTC date of an instant: "2026-11-07T23:30:00Z" -> "2026-11-07". */
export function utcDayOf(time: number): string {
  return new Date(time).toISOString().slice(0, 10);
}

const dayMonth = new Intl.DateTimeFormat("en-GB", {
  day: "numeric",
  month: "long",
  timeZone: "UTC",
});

const dayMonthYear = new Intl.DateTimeFormat("en-GB", {
  day: "numeric",
  month: "long",
  year: "numeric",
  timeZone: "UTC",
});

/** "2026-11-04" -> "4 November", or "4 November 2026" with the year. */
export function formatUtcDate(day: string, withYear = false): string {
  const format = withYear ? dayMonthYear : dayMonth;
  return unbroken(format.format(new Date(dayTime(day))));
}

/**
 * A range from its first included date to its first excluded date, shown
 * with its last included date: ("2026-10-09", "2026-11-08") ->
 * "9 October – 7 November". A range of one date shows that date.
 */
export function formatUtcRange(from: string, until: string): string {
  const last = addUtcDays(until, -1);
  if (last <= from) return formatUtcDate(from);
  return `${formatUtcDate(from)}${nbsp}– ${formatUtcDate(last)}`;
}
