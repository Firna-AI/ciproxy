// The one banner of a workspace page, from the banner that the API chose. The
// API decides which banner shows and which action the role gets; this model
// only words it. A banner says only what its fields hold: a failed payment
// names its dates when the API gives them, and an ended plan or a passed
// limit stays general, because the API names neither the plan nor the limit.
// On Plans itself, a link to Plans would only reload the page, so it opens
// Plan and usage. Only the Organizations scope gets the banner of inactive
// organizations. It gives the reasons of the API in the words of their rows,
// and offers only what each activation allows: Team, Turn on, or who to ask.

import type {
  BannerAction,
  InactiveOrganization,
  PageBanner,
} from "../api/billing-types.ts";
import type { WorkspacePage } from "../routing/pages.ts";
import {
  askPayer,
  capacityText,
  dayText,
  monthlyText,
  selectionText,
} from "./billing-format.ts";
import { billingHref, billingPaths, plansHref } from "./billing-routes.ts";
import type { NoticeTone } from "./page-model.ts";
import { holdReason } from "./settings-organizations.ts";

/**
 * What the banner offers: a page, the billing portal, who to ask, or Turn on
 * for one inactive organization. A link is primary unless it says otherwise.
 */
export type BannerActionModel =
  | { kind: "link"; label: string; href: string; primary?: false }
  | { kind: "portal"; label: string }
  | { kind: "ask"; text: string }
  | { kind: "turn-on"; label: "Turn on"; organizationId: string };

export type BannerModel = {
  tone: NoticeTone;
  title: string;
  body: string;
  actions: readonly BannerActionModel[];
};

const usageLink = (workspaceId: string): BannerActionModel => ({
  kind: "link",
  label: "Plan and usage",
  href: billingHref(workspaceId),
});

/** A link to Plans, or to Plan and usage on Plans itself. */
function plansLink(
  label: string,
  workspaceId: string,
  path: string | undefined,
): BannerActionModel {
  return path === billingPaths.plans
    ? usageLink(workspaceId)
    : { kind: "link", label, href: plansHref(workspaceId) };
}

function payment(
  action: BannerAction,
  workspaceId: string,
  path: string | undefined,
): BannerActionModel[] {
  if (action === "ask_admin_or_billing")
    return [{ kind: "ask", text: askPayer }];
  if (action === "update_card")
    return [{ kind: "portal", label: "Update card" }];
  return [plansLink("Choose a plan", workspaceId, path)];
}

/**
 * The action of a passed limit: Plan and usage, which shows the use and the
 * offer. On the other billing pages, the plans to compare.
 */
function offerAction(
  action: BannerAction,
  workspaceId: string,
  page: WorkspacePage,
  path: string | undefined,
): BannerActionModel[] {
  if (action === "ask_admin_or_billing")
    return [{ kind: "ask", text: askPayer }];
  if (action === "update_card")
    return [{ kind: "portal", label: "Update card" }];
  return page === "billing"
    ? [plansLink("Compare plans", workspaceId, path)]
    : [usageLink(workspaceId)];
}

/** The API names no other organization, so the banner stays general. */
const othersKeepRunning =
  "Jobs keep running in the other organizations of this workspace.";

/**
 * Why one inactive organization starts no jobs, and when it starts them. The
 * API names the person who added it by the GitHub ID only.
 */
function inactiveText(organization: InactiveOrganization): string {
  const { reasons } = organization.activation;
  if (reasons.length === 0) return othersKeepRunning;
  const why = reasons
    .map((reason, index) =>
      holdReason(organization, reason, organization.login, index === 0),
    )
    .join(", and ");
  const plan = reasons.includes("billing_upgrade");
  const admin = reasons.includes("admin_approval");
  const start =
    plan && admin
      ? "It starts its jobs on Team, after an Admin turns it on."
      : plan
        ? "It starts its jobs on Team."
        : "It starts its jobs after an Admin turns it on.";
  return `${why}. ${start} ${othersKeepRunning}`;
}

/**
 * What the activations of the API allow: Team for a plan hold, as the mockup
 * offers it beside Plan and usage, Turn on for the one organization that this
 * Admin may turn on, and who to ask.
 */
function inactiveActions(
  organizations: readonly InactiveOrganization[],
): BannerActionModel[] {
  const actions: BannerActionModel[] = [];
  const upgrade =
    organizations.find((item) => item.activation.upgrade)?.activation.upgrade ??
    null;
  if (upgrade !== null) {
    actions.push(
      {
        kind: "link",
        label: "Plan and usage",
        href: billingHref(upgrade.workspace_id),
        primary: false,
      },
      {
        kind: "link",
        label: "Choose Team",
        href: plansHref(upgrade.workspace_id),
      },
    );
  }
  const [only] = organizations;
  if (
    organizations.length === 1 &&
    only !== undefined &&
    only.activation.can_enable
  ) {
    actions.push({
      kind: "turn-on",
      label: "Turn on",
      organizationId: only.id,
    });
  }
  if (organizations.some((item) => item.activation.ask_admin)) {
    actions.push({ kind: "ask", text: "Ask an Admin" });
  }
  return actions;
}

export function bannerModel(
  banner: PageBanner,
  workspace: { id: string; name: string },
  page: WorkspacePage,
  /** The path of the shown page, such as "/plans". */
  path?: string,
): BannerModel {
  switch (banner.kind) {
    case "payment_failed":
      return {
        tone: "danger",
        title: banner.failed_at
          ? `Stripe could not charge your card on ${dayText(banner.failed_at)}`
          : "Stripe could not charge your card",
        body: banner.next_retry_at
          ? `Your plan stays active while Stripe tries again on ${dayText(banner.next_retry_at)}. Jobs keep running.`
          : "Your plan stays active while Stripe tries again. Jobs keep running.",
        actions: payment(banner.action, workspace.id, path),
      };
    case "subscription_ended":
      return {
        tone: "warning",
        title: "Your paid plan ended after failed payments",
        body: `${workspace.name} is on Free now. Jobs keep running on all own machines.`,
        actions: payment(banner.action, workspace.id, path),
      };
    case "soft_limit":
      return {
        tone: "warning",
        title: `${workspace.name} passed a limit of its plan`,
        body: `Jobs keep running. ${selectionText(banner.offer.selection)} covers your use for ${monthlyText(banner.offer.monthly_cents)}: ${capacityText(banner.offer)}.`,
        actions: offerAction(banner.action, workspace.id, page, path),
      };
    case "inactive_organizations": {
      const [first] = banner.organizations;
      const one = banner.organizations.length === 1 ? first : undefined;
      return {
        tone: "warning",
        title: one
          ? `${one.login} is inactive`
          : `${banner.organizations.length} organizations are inactive`,
        body: one
          ? inactiveText(one)
          : `Each row says why. ${othersKeepRunning}`,
        actions: inactiveActions(banner.organizations),
      };
    }
  }
}
