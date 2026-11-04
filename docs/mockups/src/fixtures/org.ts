// The sample organization of the mockups. mokly-ai signs up on Tuesday 1
// September 2026, passes the Free starts on Friday 4 September, moves to Team
// that day, and runs three own machines from mid-September. The November
// screens show a busy Team organization; the December screens show the
// payment problems.

/** The dashboard host shown in the address bar of every screen. */
export const host = "app.ciproxy.dev";

export const org = {
  login: "mokly-ai",
  initial: "M",
  repositories: 14,
  publicRepositories: 2,
} as const;

export const user = {
  name: "Cal Moore",
  login: "calummoore",
  initials: "CM",
} as const;

/** The address of a dashboard page of mokly-ai, for the screen `address`. */
export function orgAddress(page: string): string {
  return `${host}/${org.login}/${page}`;
}

/** The tiers the shell can show beside the organization name. */
export type TierKey = "free" | "team" | "business";
