import type { components } from "../../api/index.ts";
import { activation } from "./fixtures-core.ts";
import { pageBanner } from "./fixtures-billing-usage.ts";
import { organization } from "./fixtures-setup.ts";
import {
  pageError,
  pageReply,
  waitForPage,
  type PageReply,
  type PagesFixtureState,
} from "./harness-pages-state.ts";
import { success, type HarnessState } from "./harness-state.ts";

type Schemas = components["schemas"];
type Mode = NonNullable<PagesFixtureState["organizationBanner"]>;

export const fixtureBannerOrganizationId =
  "40000000-0000-4000-8000-000000000002";

function inactive(
  mode: Mode,
  workspaceId: string,
  role: Schemas["WorkspaceRole"],
): Schemas["InactiveOrganization"] {
  const plan = mode === "plan";
  return {
    id: fixtureBannerOrganizationId,
    login: "orbit-labs",
    activation: activation({
      admitted: false,
      added_by: "4783921",
      reasons: plan ? ["billing_upgrade"] : ["admin_approval"],
      upgrade:
        plan && role !== "viewer"
          ? { tier: "team", workspace_id: workspaceId }
          : null,
      can_enable: !plan && role === "admin",
      ask_admin: plan ? role === "viewer" : role !== "admin",
    }),
  };
}

function settings(state: HarnessState, mode: Mode): Schemas["SettingsView"] {
  return {
    name: state.scenario.session.workspaces[0]?.name ?? "Northstar",
    labels: state.scenario.setup.labels,
    organizations: [
      organization(),
      organization({
        id: fixtureBannerOrganizationId,
        login: "orbit-labs",
        github_id: "8002",
        installation_id: "9002",
        added_by: {
          id: "4783921",
          login: "anasouza",
          name: "Ana Souza",
          avatar_url: null,
        },
        // The row stays stale for payment and null banner authority checks.
        // A successful enable clears its hold so the Settings watch sees it.
        activation: activation({
          admitted: mode === "enabled",
          added_by: "4783921",
          reasons: mode === "enabled" ? [] : ["admin_approval"],
          can_enable: mode !== "enabled",
        }),
      }),
    ],
    open_requests: 0,
    can_add_organization: true,
    new_organization_requires_upgrade: false,
  };
}

export async function serveOrganizationPages(
  method: string,
  workspaceId: string,
  action: string,
  url: URL,
  state: HarnessState,
): Promise<PageReply | undefined> {
  const pages = state.scenario.pages;
  const mode = pages.organizationBanner;
  const enable = `organizations/${fixtureBannerOrganizationId}/enable`;
  if (
    mode === null ||
    (action !== "settings" && action !== "banner" && action !== enable)
  )
    return undefined;
  const role = state.scenario.session.workspaces.find(
    (item) => item.id === workspaceId,
  )?.role;
  if (role === undefined || role === null)
    return pageError(403, "access_denied", "A workspace role is required.");
  if (action !== "banner" && role !== "admin")
    return pageError(403, "access_denied", "Admin role required.");
  if (method !== (action === enable ? "POST" : "GET"))
    return pageError(405, "method_not_allowed", "Use the supported method.");
  if (action === "settings") return pageReply(settings(state, mode));
  if (action === enable) {
    if (mode === "enable-failed")
      return pageError(503, "unavailable", "Turn on is unavailable.");
    pages.organizationBanner = "enabled";
    return pageReply(success);
  }
  await waitForPage(pages, "banner");
  if (pages.failures.includes("banner"))
    return pageError(503, "unavailable", "The banner is unavailable.");
  const banner = pageBanner(
    pages.usage,
    role,
    url.searchParams.get("page") === "organizations" &&
      mode !== "none" &&
      mode !== "enabled",
  );
  if (banner?.kind === "inactive_organizations")
    banner.organizations = [inactive(mode, workspaceId, role)];
  return pageReply({ banner } satisfies Schemas["PageBannerView"]);
}
