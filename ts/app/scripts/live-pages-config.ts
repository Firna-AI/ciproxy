import type { PlanSelection } from "../src/api/billing-types.ts";
import { readSelection } from "../src/api/billing-readers.ts";
import { object, optional, string } from "../src/api/parse-core.ts";
import { readPrivateJson } from "./live-pages-private.ts";

export type RuntimeRedirects = {
  success_url: string;
  cancel_url: string;
  portal_return_url: string | null;
};

export type LivePagesCredentials = {
  base_url: string;
  workspace_id: string;
  session_cookie: string;
  csrf_token: string;
  result_path: string | null;
  selection: PlanSelection | null;
  runtime_redirects: RuntimeRedirects | null;
};

function text(value: unknown): string {
  const parsed = string(value);
  if (parsed === "" || /[\r\n]/.test(parsed))
    throw new Error("Invalid private configuration text.");
  return parsed;
}

export function readConfiguredSelection(value: unknown): PlanSelection {
  const selection = readSelection(value);
  if (selection.add_ons.start_blocks < 0 || selection.add_ons.machines < 0) {
    throw new Error("Invalid configured add-on quantity.");
  }
  return selection;
}

function redirects(value: unknown): RuntimeRedirects {
  const fields = object(value, "configured redirects");
  return {
    success_url: text(fields.success_url),
    cancel_url: text(fields.cancel_url),
    portal_return_url: optional(fields.portal_return_url, text),
  };
}

export async function readLivePagesCredentials(): Promise<
  LivePagesCredentials | undefined
> {
  const path = process.env.CIPROXY_DASHBOARD_CREDENTIALS_FILE;
  if (path === undefined || path === "") return undefined;
  const fields = object(await readPrivateJson(path), "private configuration");
  const base = new URL(text(fields.base_url));
  if (
    base.protocol !== "https:" ||
    base.username !== "" ||
    base.password !== "" ||
    base.pathname !== "/" ||
    base.search !== "" ||
    base.hash !== ""
  ) {
    throw new Error("Invalid dashboard origin.");
  }
  return {
    base_url: base.origin,
    workspace_id: text(fields.workspace_id),
    session_cookie: text(fields.session_cookie),
    csrf_token: text(fields.csrf_token),
    result_path: optional(fields.result_path, text),
    selection: optional(fields.selection, readConfiguredSelection),
    runtime_redirects: optional(fields.runtime_redirects, redirects),
  };
}
