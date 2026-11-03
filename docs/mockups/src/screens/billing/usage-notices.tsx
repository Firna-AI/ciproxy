// The warnings of Plan and usage. Limits are soft: each notice says that jobs
// keep running and offers the tier or the add-on that covers the use.

import type { ReactNode } from "react";
import { ExternalLink } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths } from "../../components/paths.ts";
import { formatCount } from "../../fixtures/format.ts";
import { org } from "../../fixtures/org.ts";
import { tiers } from "../../fixtures/tiers.ts";
import {
  paymentDates,
  type Usage,
  type UsageScene,
} from "../../fixtures/usage.ts";

/** A banner shows on every page of the organization; a notice on this page only. */
export function usageBanner(scene: UsageScene, usage: Usage): ReactNode {
  const team = tiers.team;
  switch (scene) {
    case "over-starts":
      return (
        <Notice
          tone="warning"
          title={`${org.login} passed the ${formatCount(team.starts)} starts of Team`}
          actions={
            <>
              <LinkAction to={paths.plansOnTeam} size="sm">
                Compare plans
              </LinkAction>
              <Action tone="primary" size="sm">
                Add 10,000 starts
              </Action>
            </>
          }
        >
          Jobs keep running. 10,000 more starts cost $10 a month, and Business
          includes 150,000.
        </Notice>
      );
    case "free-over":
      return (
        <Notice
          tone="warning"
          title={`${org.login} used the ${formatCount(tiers.free.starts)} starts of Free`}
          actions={
            <LinkAction to={paths.plans} tone="primary" size="sm">
              Choose Team
            </LinkAction>
          }
        >
          Jobs keep running. Team covers your use: {formatCount(team.starts)}{" "}
          starts and {team.nodes} own nodes for ${team.price} a month.
        </Notice>
      );
    case "payment-failed":
      return (
        <Notice
          tone="danger"
          title={`Stripe could not charge your card on ${paymentDates.failed}`}
          actions={
            <Action tone="primary" size="sm" icon={ExternalLink}>
              Update card
            </Action>
          }
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
          actions={
            <LinkAction to={paths.plans} tone="primary" size="sm">
              Choose a plan
            </LinkAction>
          }
        >
          {org.login} is on Free now, with {formatCount(usage.starts)} starts
          this month and {usage.nodes} own nodes. Jobs keep running.
        </Notice>
      );
    default:
      return undefined;
  }
}

export function usageNotice(scene: UsageScene, usage: Usage): ReactNode {
  if (scene === "near-limit") {
    return (
      <Notice
        tone="warning"
        title={`At this pace, ${org.login} passes ${formatCount(tiers.team.starts)} starts on ${usage.passesOn}`}
        actions={
          <Action tone="primary" size="sm">
            Add 10,000 starts
          </Action>
        }
      >
        Jobs keep running past the limit. 10,000 more starts cost $10 a month.
      </Notice>
    );
  }
  if (scene === "free" && usage.passesOn) {
    return (
      <Notice
        tone="warning"
        title={`At this pace, ${org.login} passes the ${formatCount(tiers.free.starts)} free starts on ${usage.passesOn}`}
        actions={
          <LinkAction to={paths.plans} tone="primary" size="sm">
            Compare plans
          </LinkAction>
        }
      >
        Jobs keep running past it. Team includes{" "}
        {formatCount(tiers.team.starts)} starts for ${tiers.team.price} a month.
      </Notice>
    );
  }
  if (scene === "over-nodes") {
    return (
      <Notice
        tone="warning"
        title={`${usage.nodes} own nodes on Team, which includes ${tiers.team.nodes}`}
      >
        Jobs keep running on all {usage.nodes}. The add-on below covers the
        extra node for $20 a month.
      </Notice>
    );
  }
  return undefined;
}
