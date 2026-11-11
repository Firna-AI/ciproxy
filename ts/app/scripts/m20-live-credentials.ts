import { readFile } from "node:fs/promises";

import { fields } from "./harness-state.ts";

export type DashboardCredentials = {
  base_url: string;
  workspace_id: string;
  session_cookie: string;
  csrf_token: string;
};

function credentials(value: unknown): DashboardCredentials | undefined {
  if (!fields(value)) return undefined;
  const { base_url, workspace_id, session_cookie, csrf_token } = value;
  if (
    typeof base_url !== "string" ||
    typeof workspace_id !== "string" ||
    typeof session_cookie !== "string" ||
    typeof csrf_token !== "string"
  )
    return undefined;
  if (
    [workspace_id, session_cookie, csrf_token].some(
      (item) => item === "" || /[\r\n]/.test(item),
    )
  )
    return undefined;
  try {
    const url = new URL(base_url);
    if (
      url.protocol !== "https:" ||
      url.username !== "" ||
      url.password !== "" ||
      url.pathname !== "/" ||
      url.search !== "" ||
      url.hash !== ""
    )
      return undefined;
    return { base_url: url.origin, workspace_id, session_cookie, csrf_token };
  } catch {
    return undefined;
  }
}

export async function loadCredentials(
  path: string | undefined,
): Promise<DashboardCredentials | undefined> {
  if (!path) return undefined;
  let raw: string;
  try {
    raw = await readFile(path, "utf8");
  } catch (error) {
    if (fields(error) && error.code === "ENOENT") return undefined;
    throw new Error("Private credential file unavailable", { cause: error });
  }
  const value: unknown = JSON.parse(raw);
  const config = credentials(value);
  if (config === undefined) throw new Error("Invalid private credentials");
  return config;
}
