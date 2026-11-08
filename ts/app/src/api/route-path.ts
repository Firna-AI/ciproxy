// The path of a route of the generated contract, with its parameters filled
// in. The route is a key of the generated paths, so a renamed route fails the
// type check, and each named parameter must be given. IDs are opaque, so each
// value is encoded as one path segment.

import type { paths } from "../../../api/index.ts";

export type Route = keyof paths;

/** The names in braces of a route, such as "workspace_id". */
type RouteParams<R extends string> =
  R extends `${string}{${infer Name}}${infer Rest}`
    ? Name | RouteParams<Rest>
    : never;

export function routePath<R extends Route>(
  route: R,
  params: Readonly<Record<RouteParams<R>, string>>,
): string {
  const values: Readonly<Record<string, string>> = params;
  return route.replace(/\{(\w+)\}/g, (_match, name: string) =>
    encodeURIComponent(values[name] ?? ""),
  );
}

/**
 * The path with a query of the given values. An absent or empty value is left
 * out, so the API applies its own default.
 */
export function withQuery(
  path: string,
  query: Readonly<Record<string, string | undefined>>,
): string {
  const search = new URLSearchParams();
  for (const [name, value] of Object.entries(query)) {
    if (value !== undefined && value !== "") search.set(name, value);
  }
  const text = search.toString();
  return text === "" ? path : `${path}?${text}`;
}
