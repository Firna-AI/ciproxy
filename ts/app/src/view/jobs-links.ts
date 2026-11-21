// The addresses of Jobs and of one job, and the History filters in them. A
// filter lives in the address, so a reload or a shared link keeps it. The
// date choice counts UTC days back from today, as the API counts them, and a
// value that the page does not know falls back to its default.

import type { HistoryPlace, HistoryQuery } from "../api/jobs-types.ts";
import { workspaceHref } from "../routing/pages.ts";
import { addUtcDays, utcDayOf } from "./exact-format.ts";

export type JobsTab = "live" | "history";

/** The history windows of the tiers, in UTC days, with Today as 1. */
export const historyWindows = [1, 7, 30, 90] as const;

export type HistoryWindow = (typeof historyWindows)[number];

export type HistoryFilters = {
  search: string;
  place: HistoryPlace | null;
  days: HistoryWindow;
};

export const defaultFilters: HistoryFilters = {
  search: "",
  place: null,
  days: 1,
};

export const windowLabels: Readonly<Record<HistoryWindow, string>> = {
  1: "Today",
  7: "Last 7 days",
  30: "Last 30 days",
  90: "Last 90 days",
};

/** The choices of the date menu: Today and each window up to the history. */
export function windowChoices(historyDays: number): HistoryWindow[] {
  return historyWindows.filter((days) => days === 1 || days <= historyDays);
}

/** The address of Live, or of History with its filters. */
export function jobsHref(
  workspaceId: string,
  tab: JobsTab,
  filters: HistoryFilters = defaultFilters,
): string {
  const base = workspaceHref("jobs", workspaceId);
  if (tab === "live") return base;
  const query = new URLSearchParams({ tab: "history" });
  const search = filters.search.trim();
  if (search !== "") query.set("search", search);
  if (filters.place !== null) query.set("place", filters.place);
  if (filters.days !== 1) query.set("days", String(filters.days));
  return `${base}&${query.toString()}`;
}

/** The address of one job, in the shell of its workspace. */
export function jobHref(workspaceId: string, jobId: string): string {
  const query = new URLSearchParams({ workspace: workspaceId, job: jobId });
  return `/job?${query.toString()}`;
}

/** One value of the query, or null for a missing or repeated one. */
function single(value: string | readonly string[] | undefined): string | null {
  return typeof value === "string" ? value : null;
}

export type JobsParams = {
  tab?: string | string[];
  search?: string | string[];
  place?: string | string[];
  days?: string | string[];
};

export function tabOf(params: JobsParams): JobsTab {
  return single(params.tab) === "history" ? "history" : "live";
}

export function filtersOf(params: JobsParams): HistoryFilters {
  const place = single(params.place);
  const days = Number(single(params.days));
  return {
    search: single(params.search) ?? "",
    place: place === "machine" || place === "boat" ? place : null,
    days: historyWindows.find((window) => window === days) ?? 1,
  };
}

/**
 * The API query of the filters at a time: the first included UTC date, and
 * the search and the place when they are set. No last date, so a job of
 * today always matches, also when the clock of the browser is behind.
 */
export function historyQuery(
  filters: HistoryFilters,
  now: number,
): HistoryQuery {
  const search = filters.search.trim();
  return {
    from: addUtcDays(utcDayOf(now), 1 - filters.days),
    search: search === "" ? undefined : search,
    place: filters.place ?? undefined,
  };
}

/** Whether a filter differs from its default, so the list can be cleared. */
export function filtered(filters: HistoryFilters): boolean {
  return (
    filters.search.trim() !== "" || filters.place !== null || filters.days !== 1
  );
}
