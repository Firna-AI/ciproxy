// The banner that every page of the organization shows while a soft limit is
// passed or a payment failed. One state shows one banner on every page, with
// the same title, text, and actions. On Plan and usage itself, the link to
// that page drops out, and an add-on action saves in place. A person who
// cannot pay sees the same banner with words in place of its actions.

import type { ReactNode } from "react";
import { ExternalLink } from "lucide-react";

import { formatCount, monthly } from "../fixtures/format.ts";
import { org } from "../fixtures/org.ts";
import {
  addOnPrices,
  ownMachines,
  startsText,
  tiers,
} from "../fixtures/tiers.ts";
import {
  paymentDates,
  usageScenes,
  type UsageScene,
} from "../fixtures/usage.ts";
import { Action, LinkAction } from "./actions.tsx";
import { Notice } from "./feedback.tsx";
import { paths, type CataloguePath } from "./paths.ts";

/** The page that shows the banner: Plan and usage, or any other page. */
export type BannerPage = "billing" | "other";

/** What a person who cannot pay sees in place of a payment action. */
export function AskPayer() {
  return <span className="caption">Ask an Admin or a Billing person</span>;
}

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
          title={`${org.login} used the ${formatCount(free.starts)} starts of Free`}
          actions={act(
            <>
              {page === "billing" ? null : (
                <LinkAction to={paths.usageFreeOver} size="sm">
                  Plan and usage
                </LinkAction>
              )}
              <LinkAction to={paths.plans} tone="primary" size="sm">
                Choose Team
              </LinkAction>
            </>,
          )}
        >
          Jobs keep running. Team covers your use for {monthly(team.price)}:{" "}
          {startsText(team.starts)} a month and {ownMachines(team.machines)}.
        </Notice>
      );
    case "over-starts":
      return (
        <Notice
          tone="warning"
          title={`${org.login} passed the ${formatCount(team.starts)} starts of Team`}
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
          {org.login} is on Free now: {startsText(free.starts)} a month and{" "}
          {ownMachines(free.machines)}. Jobs keep running on all{" "}
          {usage.machines} machines.
        </Notice>
      );
    default:
      return undefined;
  }
}
