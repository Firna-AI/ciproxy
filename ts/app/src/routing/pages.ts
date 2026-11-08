// The pages of the dashboard. Every page has a static path, so the static
// export serves each one as a file. A workspace page names its workspace in
// the `workspace` query parameter, by the stable ID, so a rename never breaks
// a link.

export const pages = {
  home: "/",
  signIn: "/sign-in",
  install: "/install",
  personalAccount: "/personal-account",
  appRemoved: "/app-removed",
  accessEnded: "/access-ended",
  addOrganization: "/add-organization",
} as const;

export const workspacePages = [
  "setup",
  "jobs",
  "machines",
  "savings",
  "billing",
  "settings",
] as const;

export type WorkspacePage = (typeof workspacePages)[number];

/** Why sign-in shows: GitHub did not finish, or the sign-in ended. */
export type SignInReason = "github" | "ended";

export function signInHref(reason: SignInReason | null): string {
  return reason === null ? pages.signIn : `${pages.signIn}?reason=${reason}`;
}

export function workspaceHref(
  page: WorkspacePage,
  workspaceId: string,
): string {
  return `/${page}?workspace=${encodeURIComponent(workspaceId)}`;
}

/** The landing of one workspace, which picks Setup or Jobs for it. */
export function workspaceHomeHref(workspaceId: string): string {
  return `${pages.home}?workspace=${encodeURIComponent(workspaceId)}`;
}

/**
 * A query value that names one ID, or null for a missing, empty, or repeated
 * value. IDs are opaque, so the caller checks the ID against the session.
 */
export function idParam(
  value: string | readonly string[] | undefined,
): string | null {
  return typeof value === "string" && value.trim() !== "" ? value : null;
}

/** The sign-in reason of the query, or null for none or an unknown one. */
export function signInReasonParam(
  value: string | readonly string[] | undefined,
): SignInReason | null {
  return value === "github" || value === "ended" ? value : null;
}
