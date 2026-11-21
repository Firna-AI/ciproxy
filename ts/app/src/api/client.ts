// The one HTTP client of the dashboard. It calls the same-origin API under
// /api with the browser's credentials, records the API build of every
// response, adds the CSRF token of the current session to each change, and
// turns every outcome into a typed result. It never throws for an HTTP or a
// network failure, so no caller can hide one by accident.

import type { BuildGate } from "../state/build-gate.ts";
import { parseProblem } from "./parse-session.ts";
import type { Problem } from "./schema.ts";

export const apiPrefix = "/api";

export type ApiFailure =
  | { kind: "problem"; status: number; problem: Problem }
  | { kind: "malformed"; status: number }
  | { kind: "network" };

export type ApiResult<T> =
  { ok: true; value: T } | { ok: false; failure: ApiFailure };

export type Transport = (url: string, init: RequestInit) => Promise<Response>;

export type ChangeMethod = "POST" | "PUT" | "DELETE";

export type ApiClient = {
  read<T>(
    path: string,
    parse: (value: unknown) => T | undefined,
  ): Promise<ApiResult<T>>;
  change<T>(
    method: ChangeMethod,
    path: string,
    parse: (value: unknown) => T | undefined,
    body?: unknown,
  ): Promise<ApiResult<T>>;
};

export type ApiClientOptions = {
  transport: Transport;
  gate: BuildGate;
  /** The CSRF token of the current session, read at the time of each call. */
  csrf: () => string | null;
  /**
   * Reload the document. The client calls it in place of the first read of
   * a page that opened after the API build changed.
   */
  reload?: () => void;
};

/** The HTTP status of a failure, or null when no response arrived. */
export function failureStatus(failure: ApiFailure): number | null {
  return failure.kind === "network" ? null : failure.status;
}

/** A sign-in that is absent, ended, or revoked. */
export function signInRequired(failure: ApiFailure): boolean {
  return failureStatus(failure) === 401;
}

/** A promise for a call that a full-document navigation replaced. */
function abandoned<T>(): Promise<T> {
  return new Promise<T>(() => undefined);
}

async function body(response: Response): Promise<unknown> {
  const text = await response.text().catch(() => "");
  if (text === "") return undefined;
  try {
    return JSON.parse(text);
  } catch {
    return undefined;
  }
}

export function createApiClient({
  transport,
  gate,
  csrf,
  reload,
}: ApiClientOptions): ApiClient {
  async function call<T>(
    method: "GET" | ChangeMethod,
    path: string,
    parse: (value: unknown) => T | undefined,
    payload: unknown,
  ): Promise<ApiResult<T>> {
    if (gate.reloading()) return abandoned();
    if (gate.leftPage()) {
      gate.beginReload();
      reload?.();
      return abandoned();
    }
    const headers: Record<string, string> = { Accept: "application/json" };
    if (method !== "GET") {
      const token = csrf();
      if (token !== null) headers["x-ciproxy-csrf"] = token;
    }
    if (payload !== undefined) headers["Content-Type"] = "application/json";
    let response: Response;
    try {
      response = await transport(apiPrefix + path, {
        method,
        headers,
        credentials: "same-origin",
        cache: "no-store",
        body: payload === undefined ? undefined : JSON.stringify(payload),
      });
    } catch {
      return { ok: false, failure: { kind: "network" } };
    }
    gate.observe(response.headers.get("x-ciproxy-build"));
    const json = await body(response);
    if (gate.reloading()) return abandoned();
    const status = response.status;
    if (response.ok) {
      const value = parse(json);
      return value === undefined
        ? { ok: false, failure: { kind: "malformed", status } }
        : { ok: true, value };
    }
    const problem = parseProblem(json);
    return {
      ok: false,
      failure:
        problem === undefined
          ? { kind: "malformed", status }
          : { kind: "problem", status, problem },
    };
  }

  return {
    read: (path, parse) => call("GET", path, parse, undefined),
    change: (method, path, parse, payload) =>
      call(method, path, parse, payload),
  };
}
