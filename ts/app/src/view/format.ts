// One format for each kind of value that the screens show. Times show in
// the time zone of the browser unless a caller names one, as tests do. A
// measured CPU or memory figure has one decimal; a size stays whole.

import type { WorkspaceRole } from "../api/schema.ts";
import { formatCount, unbroken } from "./exact-format.ts";

export const roleNames: Readonly<Record<WorkspaceRole, string>> = {
  admin: "Admin",
  billing: "Billing",
  viewer: "Viewer",
};

/** "20 December at 10:00": a deadline to the minute. */
export function formatDeadline(iso: string, timeZone?: string): string {
  return new Intl.DateTimeFormat("en-GB", {
    day: "numeric",
    month: "long",
    hour: "2-digit",
    minute: "2-digit",
    timeZone,
  }).format(new Date(iso));
}

/** "12 November": the day of an instant, kept on one line. */
export function formatDay(iso: string, timeZone?: string): string {
  return new Intl.DateTimeFormat("en-GB", {
    day: "numeric",
    month: "long",
    timeZone,
  })
    .format(new Date(iso))
    .replace(" ", "\u00a0");
}

/**
 * "1 September": a day in the time zone of the browser, with an ordinary
 * space, as the Members and Organizations tabs of Settings show it.
 */
export function formatPlainDay(iso: string, timeZone?: string): string {
  return new Intl.DateTimeFormat("en-GB", {
    day: "numeric",
    month: "long",
    timeZone,
  }).format(new Date(iso));
}

/** "6m 12s", "45s", or "1h 4m": the time between two instants. */
export function formatDuration(
  startIso: string,
  endIso: string,
): string | null {
  const seconds = Math.round(
    (Date.parse(endIso) - Date.parse(startIso)) / 1000,
  );
  if (!Number.isFinite(seconds) || seconds < 0) return null;
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const rest = seconds % 60;
  if (hours > 0) return `${hours}h ${minutes}m`;
  if (minutes > 0) return `${minutes}m ${rest}s`;
  return `${rest}s`;
}

/** A byte count in whole decimal gigabytes: "64000000000" -> 64. */
export function wholeGigabytes(bytes: string): number {
  return Math.round(Number(bytes) / 1e9);
}

/** A byte count as whole decimal gigabytes, such as "64 GB". */
export function formatGigabytes(bytes: string): string {
  return `${wholeGigabytes(bytes)} GB`;
}

/** A whole amount in tenths of its unit, half up: (1550n, 100n) -> "1.6". */
function tenths(amount: bigint, tenth: bigint): string {
  const rounded = (amount * 2n + tenth) / (tenth * 2n);
  return `${formatCount(rounded / 10n)}.${rounded % 10n}`;
}

/**
 * "1.6 vCPU": a measured CPU figure from thousandths of a vCPU, to one
 * place, half up, on one line. A VM size or a label size stays whole.
 */
export function formatMeasuredVcpu(milliVcpu: number): string {
  return unbroken(`${tenths(BigInt(milliVcpu), 100n)} vCPU`);
}

/**
 * "6.1 GB": a measured memory figure from bytes, in decimal gigabytes to one
 * place, half up, on one line. The digits never pass through a float.
 */
export function formatMeasuredGigabytes(bytes: string): string {
  return unbroken(`${tenths(BigInt(bytes), 100_000_000n)} GB`);
}

/** "1 slot", "6 slots". */
export function countText(count: number, one: string, many: string): string {
  return `${count} ${count === 1 ? one : many}`;
}

/** "a", "a or b", "a, b or c". */
export function joinWords(
  items: readonly string[],
  word: "and" | "or",
): string {
  if (items.length <= 1) return items[0] ?? "";
  return `${items.slice(0, -1).join(", ")} ${word} ${items[items.length - 1]}`;
}

/** The letter of an avatar for a login or a workspace name. */
export function initial(name: string): string {
  return (name.trim()[0] ?? "?").toUpperCase();
}

export function signedInAs(login: string): string {
  return `Signed in as ${login} on GitHub.`;
}

/** "14:32": the time of an instant, to the minute. */
export function formatTime(iso: string, timeZone?: string): string {
  return new Intl.DateTimeFormat("en-GB", {
    hour: "2-digit",
    minute: "2-digit",
    timeZone,
  }).format(new Date(iso));
}
