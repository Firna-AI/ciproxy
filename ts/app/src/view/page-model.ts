// The shape of a page in the signed-out frame: a title, a lead, one notice,
// an optional list, and the actions in order of weight. Screens render the
// model; tests read it.

export type ActionTone = "primary" | "secondary";

export type PageAction =
  /** A top-level navigation that leaves the app, such as GitHub. */
  | {
      kind: "leave";
      label: string;
      href: string;
      tone: ActionTone;
      icon: "external" | null;
      newTab: boolean;
    }
  /** An explicit start saves installation evidence before opening GitHub. */
  | { kind: "install"; label: string; href: string; tone: ActionTone }
  /** The server-owned check: fresh GitHub proof or a new read. */
  | { kind: "check-again"; label: string; tone: ActionTone }
  | {
      kind: "claim-admin";
      label: string;
      tone: ActionTone;
      workspaceId: string;
    }
  /** A page of the app. */
  | { kind: "go"; label: string; href: string; tone: ActionTone };

export type NoticeTone = "info" | "warning" | "danger";

export type NoticeModel = { tone: NoticeTone; title: string; body: string };

export type ListModel = { caption: string; items: readonly string[] };

export type AuthPageModel = {
  title: string;
  lead: string | null;
  notice: NoticeModel | null;
  list: ListModel | null;
  actions: readonly PageAction[];
  footer: string;
};

export const siteLinks = {
  terms: "https://ciproxy.dev/terms",
  privacy: "https://ciproxy.dev/privacy",
  docs: "https://ciproxy.dev/docs",
} as const;

/** A top-level link out of the app; an external one opens a new tab. */
export function leave(
  label: string,
  href: string,
  tone: ActionTone,
  icon: "external" | null = null,
): PageAction {
  const newTab = icon === "external";
  return { kind: "leave", label, href, tone, icon, newTab };
}

/** Start installation, with a stable navigation target for another tab. */
export function startInstall(
  label: string,
  href: string,
  tone: ActionTone,
): PageAction {
  return { kind: "install", label, href, tone };
}
