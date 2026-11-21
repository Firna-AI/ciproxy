// The small readers that turn an unknown response body into a generated type.
// Each reader throws a ParseError on a wrong shape, and `parser` turns that
// into `undefined` at the boundary, so a screen never sees a partial value.

export class ParseError extends Error {
  constructor(what: string) {
    super(`The response has an invalid ${what}.`);
    this.name = "ParseError";
  }
}

export type Fields = Readonly<Record<string, unknown>>;

function isFields(value: unknown): value is Fields {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** Wrap a throwing reader into one that returns undefined on a wrong shape. */
export function parser<T>(
  read: (value: unknown) => T,
): (value: unknown) => T | undefined {
  return (value) => {
    try {
      return read(value);
    } catch (error) {
      if (error instanceof ParseError) return undefined;
      throw error;
    }
  };
}

export function object(value: unknown, what: string): Fields {
  if (!isFields(value)) throw new ParseError(what);
  return value;
}

export function string(value: unknown): string {
  if (typeof value !== "string") throw new ParseError("text value");
  return value;
}

export function boolean(value: unknown): boolean {
  if (typeof value !== "boolean") throw new ParseError("flag");
  return value;
}

export function integer(value: unknown): number {
  if (typeof value !== "number" || !Number.isInteger(value)) {
    throw new ParseError("whole number");
  }
  return value;
}

/** An integer in decimal-string form, such as a byte count or cents. */
export function decimal(value: unknown): string {
  const text = string(value);
  if (!/^-?\d+$/.test(text)) throw new ParseError("decimal string");
  return text;
}

/**
 * An exact decimal in string form, with an optional fraction, such as
 * "372.5" seconds or "80604.25" minutes. It stays a string, so no digit is
 * lost.
 */
export function fraction(value: unknown): string {
  const text = string(value);
  if (!/^-?\d+(\.\d+)?$/.test(text)) throw new ParseError("decimal value");
  return text;
}

/** A UTC calendar date in YYYY-MM-DD form, kept as its wire string. */
export function date(value: unknown): string {
  const text = string(value);
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(text);
  if (match === null) throw new ParseError("date");
  const [year, month, day] = [match[1], match[2], match[3]].map(Number);
  const at = new Date(Date.UTC(year ?? 0, (month ?? 1) - 1, day ?? 1));
  if (at.toISOString().slice(0, 10) !== text) throw new ParseError("date");
  return text;
}

/** A UTC RFC 3339 time, kept as its wire string. */
export function time(value: unknown): string {
  const text = string(value);
  if (!Number.isFinite(Date.parse(text))) throw new ParseError("time");
  return text;
}

/** An absent or null value becomes null; any other value must parse. */
export function optional<T>(
  value: unknown,
  read: (value: unknown) => T,
): T | null {
  return value === undefined || value === null ? null : read(value);
}

export function array<T>(value: unknown, read: (value: unknown) => T): T[] {
  if (!Array.isArray(value)) throw new ParseError("list");
  return value.map((item: unknown) => read(item));
}

function hasKey<T extends string>(
  members: Readonly<Record<T, true>>,
  value: string,
): value is T {
  return Object.prototype.hasOwnProperty.call(members, value);
}

/**
 * A reader for a closed set of strings. The record must name every member of
 * the generated union and nothing else, so the type check fails when the
 * contract gains or loses a member.
 */
export function oneOf<T extends string>(
  members: Readonly<Record<T, true>>,
): (value: unknown) => T {
  return (value) => {
    if (typeof value === "string" && hasKey(members, value)) return value;
    throw new ParseError("state");
  };
}
