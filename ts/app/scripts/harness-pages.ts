import type { components } from "../../api/index.ts";
import { historyDays, plansPage } from "./fixtures-billing.ts";
import { pageBanner, usageForRole } from "./fixtures-billing-usage.ts";
import { filteredHistory, visibleHistory } from "./fixtures-jobs.ts";
import { liveJobs } from "./fixtures-jobs-live.ts";
import { savingsRanges, savingsReport } from "./fixtures-savings.ts";
import { serveBillingPage } from "./harness-pages-billing.ts";
import {
  pageError,
  pageReply,
  pageRoutes,
  waitForPage,
  type PageReply,
  type PageRoute,
} from "./harness-pages-state.ts";
import type { HarnessState } from "./harness-state.ts";

function identifyRoute(action: string): PageRoute | undefined {
  const exact = pageRoutes.find(
    (route) => route !== "jobs/detail" && route === action,
  );
  return exact ?? (/^jobs\/[^/]+$/.test(action) ? "jobs/detail" : undefined);
}

export async function servePages(
  method: string,
  workspaceId: string,
  action: string,
  url: URL,
  state: HarnessState,
  readBody: () => Promise<unknown>,
): Promise<PageReply | undefined> {
  const route = identifyRoute(action);
  if (route === undefined) return undefined;
  const mutation =
    route === "checkout" ||
    route === "checkout/return" ||
    route.startsWith("billing/");
  const expectedMethod = mutation
    ? route === "billing/add-ons"
      ? "PUT"
      : "POST"
    : "GET";
  const initial = state.scenario.session.workspaces.find(
    (item) => item.id === workspaceId,
  );
  if (initial?.role === undefined || initial.role === null)
    return pageError(403, "access_denied", "A workspace role is required.");
  if ((mutation || route === "checkout/review") && initial.role === "viewer")
    return pageError(
      403,
      "access_denied",
      "An Admin or Billing role is required.",
    );
  if (method !== expectedMethod)
    return pageError(
      405,
      "method_not_allowed",
      "Use the supported request method.",
    );
  await waitForPage(state.scenario.pages, route);
  const member = state.scenario.session.workspaces.find(
    (item) => item.id === workspaceId,
  );
  if (state.scenario.signedOut)
    return pageError(401, "sign_in_required", "Sign in again.");
  if (
    state.scenario.session.state !== "ready" ||
    member?.role === undefined ||
    member.role === null ||
    ((mutation || route === "checkout/review") && member.role === "viewer")
  )
    return pageError(403, "access_denied", "Access ended.");
  const pages = state.scenario.pages;
  if (mutation || route === "checkout/review")
    return serveBillingPage(
      route,
      url.searchParams,
      mutation ? await readBody() : undefined,
      pages,
      member.role,
    );
  if (pages.failures.includes(route))
    return pageError(503, "unavailable", "The page is unavailable. Try again.");
  const usage = usageForRole(pages.usage, member.role);
  const banner = usage.banner ?? null;
  const window = historyDays(usage.status.selection.tier);
  switch (route) {
    case "banner":
      return pageReply({
        banner: pageBanner(
          pages.usage,
          member.role,
          pages.inactiveOrganizations &&
            url.searchParams.get("page") === "organizations",
        ),
      } satisfies components["schemas"]["PageBannerView"]);
    case "jobs/live":
      return pageReply(liveJobs(pages.jobs, member.role, banner));
    case "jobs/history":
      return pageReply({
        banner,
        history_days: window,
        jobs:
          pages.history === "empty"
            ? []
            : filteredHistory(window, url.searchParams),
      } satisfies components["schemas"]["JobsHistoryPage"]);
    case "jobs/detail": {
      const id = decodeURIComponent(action.slice("jobs/".length));
      const live = liveJobs(pages.jobs, member.role, banner);
      const job = [
        ...visibleHistory(window),
        ...live.running,
        ...live.waiting,
      ].find((item) => item.id === id);
      if (pages.detail === "not-found" || job === undefined)
        return pageError(
          404,
          "not_found",
          "Job not found in this workspace and history window.",
        );
      return pageReply({
        job,
        banner,
      } satisfies components["schemas"]["JobDetailPage"]);
    }
    case "savings": {
      const range = url.searchParams.has("range")
        ? savingsRanges.find((item) => item === url.searchParams.get("range"))
        : pages.range;
      if (range === undefined)
        return pageError(
          400,
          "invalid_request",
          "Choose a supported UTC range.",
        );
      return pageReply(
        savingsReport(pages.savings, range, member.role === "admin", banner),
      );
    }
    case "billing":
      return pageReply(usage);
    case "plans":
      return pageReply(plansPage(usage));
    default:
      return pageError(404, "not_found", "Page not found.");
  }
}
