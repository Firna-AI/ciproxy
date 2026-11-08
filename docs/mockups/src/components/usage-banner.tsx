// The banner that every page of the workspace shows while a soft limit is
// passed or a payment failed. One state shows one banner on every page, with
// the same title, text, and actions. At a failed payment, the payment banner
// takes the place of the soft-limit banner, as design call 42 says. On Free,
// one organization is a soft limit too, so a workspace that keeps more
// organizations after a move to Free sees the banner and the offer. On Plan
// and usage itself, the link to that page drops out, and an add-on action
// saves in place. A person who cannot pay sees the same banner with words in
// place of its actions. The banner of the organizations takes the pages of
// its moment, so that its links keep the organizations of that moment.

import type { ReactNode } from "react";
import { ExternalLink } from "lucide-react";

import { formatCount, monthly } from "../fixtures/format.ts";
import { organizationList, organizations, workspace } from "../fixtures/org.ts";
import {
  addOnPrices,
  ownMachines,
  startsText,
  tiers,
} from "../fixtures/tiers.ts";
import {
  paymentDates,
  usageScenes,
  type Usage,
  type UsageScene,
} from "../fixtures/usage.ts";
import { Action, LinkAction } from "./actions.tsx";
import { Notice } from "./feedback.tsx";
import { paths, type CataloguePath } from "./paths.ts";
import { AskPayer } from "./role-actions.tsx";

/** The page that shows the banner: Plan and usage, or any other page. */
export type BannerPage = "billing" | "other";

/** The Plan and usage and the Plans screens of one moment of the story. */
export type BillingPages = { usage: CataloguePath; plans: CataloguePath };

/** An add-on action: in place on Plan and usage, a link from other pages. */
function AddOnAction({
  page,
  to,
  children,
}: {
  page: BannerPage;
  to: CataloguePath;
  children: string;
}) {
  return page === "billing" ? (
    <Action tone="primary" size="sm">
      {children}
    </Action>
  ) : (
    <LinkAction to={to} tone="primary" size="sm">
      {children}
    </LinkAction>
  );
}

/** The offer of Team on Free, with the link to Plan and usage elsewhere. */
export function ChooseTeam({
  page,
  usage,
  plans = paths.plans,
}: {
  page: BannerPage;
  /** The Plan and usage screen of the state. */
  usage: CataloguePath;
  /** The Plans screen of the state. */
  plans?: CataloguePath;
}) {
  return (
    <>
      {page === "billing" ? null : (
        <LinkAction to={usage} size="sm">
          Plan and usage
        </LinkAction>
      )}
      <LinkAction to={plans} tone="primary" size="sm">
        Choose Team
      </LinkAction>
    </>
  );
}

/**
 * The banner of more organizations than Free includes. It counts every
 * linked organization, also an inactive one, and names those that run.
 */
export function organizationsBanner({
  count,
  usage,
  pages,
  page,
  canPay = true,
}: {
  count: number;
  usage: Usage;
  pages: BillingPages;
  page: BannerPage;
  canPay?: boolean;
}): ReactNode {
  const { free, team } = tiers;
  return (
    <Notice
      tone="warning"
      title={`${count} organizations on Free, which includes ${free.organizations ?? 1}`}
      actions={canPay ? <ChooseTeam page={page} {...pages} /> : <AskPayer />}
    >
      Jobs keep running in {organizationList("and")}, on all {usage.machines}{" "}
      machines. Team covers your use for {monthly(team.price)}: several
      organizations, {startsText(team.starts)} a month, and{" "}
      {ownMachines(team.machines)}.
    </Notice>
  );
}

export function usageBanner(
  scene: UsageScene,
  page: BannerPage,
  canPay = true,
): ReactNode {
  const usage = usageScenes[scene];
  const { free, team, business } = tiers;
  const act = (actions: ReactNode) => (canPay ? actions : <AskPayer />);
  switch (scene) {
    case "free-over":
      return (
        <Notice
          tone="warning"
          title={`${workspace.name} used the ${formatCount(free.starts)} starts of Free`}
          actions={act(<ChooseTeam page={page} usage={paths.usageFreeOver} />)}
        >
          Jobs keep running. Team covers your use for {monthly(team.price)}:{" "}
          {startsText(team.starts)} a month and {ownMachines(team.machines)}.
        </Notice>
      );
    case "free-organizations":
      return organizationsBanner({
        count: organizations.length,
        usage,
        pages: { usage: paths.freeOrganizations, plans: paths.plans },
        page,
        canPay,
      });
    case "over-starts":
      return (
        <Notice
          tone="warning"
          title={`${workspace.name} passed the ${formatCount(team.starts)} starts of Team`}
          actions={act(
            <>
              <LinkAction to={paths.plansOnTeam} size="sm">
                Compare plans
              </LinkAction>
              <AddOnAction page={page} to={paths.usageOverStarts}>
                {`Add ${formatCount(addOnPrices.startsBlock)} starts`}
              </AddOnAction>
            </>,
          )}
        >
          Jobs keep running. {formatCount(addOnPrices.startsBlock)} more starts
          cost {monthly(addOnPrices.startsPrice)}, and Business includes{" "}
          {formatCount(business.starts)}.
        </Notice>
      );
    case "over-machines":
      return (
        <Notice
          tone="warning"
          title={`${ownMachines(usage.machines)} on Team, which includes ${team.machines}`}
          actions={act(
            <AddOnAction page={page} to={paths.usageOverMachines}>
              Add an extra own machine
            </AddOnAction>,
          )}
        >
          Jobs keep running on all {usage.machines}. An extra own machine costs{" "}
          {monthly(addOnPrices.machinePrice)}.
        </Notice>
      );
    case "payment-failed":
    case "payment-failed-over-limit":
      return (
        <Notice
          tone="danger"
          title={`Stripe could not charge your card on ${paymentDates.failed}`}
          actions={act(
            <Action tone="primary" size="sm" icon={ExternalLink}>
              Update card
            </Action>,
          )}
        >
          Team stays active while Stripe tries again on {paymentDates.retry}.
          Jobs keep running.
        </Notice>
      );
    case "moved-to-free":
      return (
        <Notice
          tone="warning"
          title={`Team ended on ${paymentDates.ended} after failed payments`}
          actions={act(
            <LinkAction to={paths.plans} tone="primary" size="sm">
              Choose a plan
            </LinkAction>,
          )}
        >
          {workspace.name} is on Free now: {startsText(free.starts)} a month and{" "}
          {ownMachines(free.machines)}. Jobs keep running on all{" "}
          {usage.machines} machines.
        </Notice>
      );
    default:
      return undefined;
  }
}
