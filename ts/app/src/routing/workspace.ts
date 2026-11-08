// The access of the signed-in person to one workspace page. A page with a
// missing or unknown workspace goes back to the landing. A person with no
// role sees the page that asks for access in place of every page. Setup and
// Settings are for Admins only.

import type {
  SessionView,
  WorkspaceRole,
  WorkspaceView,
} from "../api/schema.ts";
import { pages, workspaceHref, type WorkspacePage } from "./pages.ts";

export type WorkspaceAccess =
  | { kind: "redirect"; href: string }
  | { kind: "no-role"; workspace: WorkspaceView }
  | { kind: "member"; workspace: WorkspaceView; role: WorkspaceRole };

const adminPages: ReadonlySet<WorkspacePage> = new Set(["setup", "settings"]);

export function workspaceAccess(
  session: SessionView,
  workspaceId: string | null,
  page: WorkspacePage,
): WorkspaceAccess {
  if (session.state !== "ready") return { kind: "redirect", href: pages.home };
  const workspace = session.workspaces.find((item) => item.id === workspaceId);
  if (workspace === undefined) return { kind: "redirect", href: pages.home };
  const role = workspace.role ?? null;
  if (role === null) return { kind: "no-role", workspace };
  if (adminPages.has(page) && role !== "admin") {
    return { kind: "redirect", href: workspaceHref("jobs", workspace.id) };
  }
  return { kind: "member", workspace, role };
}
