import type { components } from "../../api/index.ts";

export type CapturedBody =
  | null
  | components["schemas"]["PlanSelection"]
  | components["schemas"]["PlanAddOns"]
  | components["schemas"]["CheckoutReturn"]
  | { invalid: true };

export function queryFacts(url: URL): Record<string, string[]> {
  const safe = [
    "workspace",
    "search",
    "place",
    "from",
    "until",
    "range",
    "tier",
    "start_blocks",
    "machines",
    "page",
  ];
  const result: Record<string, string[]> = {};
  for (const key of new Set(url.searchParams.keys())) {
    result[key] = url.searchParams
      .getAll(key)
      .map((value) => (safe.includes(key) ? value : "[redacted]"));
  }
  return result;
}
