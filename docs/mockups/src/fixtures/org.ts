// The sample organization of the mockups. The story runs through November
// 2026: mokly-ai signs up on 1 November, passes the Free starts on 4 November,
// moves to Team, and runs three own machines by mid-month.

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
