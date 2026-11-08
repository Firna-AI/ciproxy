// The notices of Plan and usage that only this page shows: the pace of the
// starts passes the tier before the period ends, or a paid tier ends after a
// move to Free. Limits are soft, so each notice says that jobs keep running
// and offers what covers the use. The banners of a passed limit or a failed
// payment show on every page (components/usage-banner.tsx). On Free, the use
// decides the pace notice: every moment whose starts are below the included
// starts and whose expected starts pass them shows it, below any banner.

import type { ReactNode } from "react";
import { ExternalLink } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import type { CataloguePath } from "../../components/paths.ts";
import { AskPayer } from "../../components/role-actions.tsx";
import { formatCount, monthly } from "../../fixtures/format.ts";
import { workspace } from "../../fixtures/org.ts";
import { addOnPrices, startsText, tiers } from "../../fixtures/tiers.ts";
import {
  freeFrom,
  paceCrossing,
  type Usage,
  type UsageScene,
} from "../../fixtures/usage.ts";

export function usageNotice(
  scene: UsageScene,
  usage: Usage,
  canPay: boolean,
  /** The Plans screen of the moment, which Compare plans opens. */
  plans: CataloguePath,
): ReactNode {
  const block = formatCount(addOnPrices.startsBlock);
  const act = (actions: ReactNode) => (canPay ? actions : <AskPayer />);
  const freePassesOn = usage.tier === "free" ? paceCrossing(usage) : undefined;
  if (scene === "near-limit") {
    return (
      <Notice
        tone="warning"
        title={`At this pace, ${workspace.name} passes ${formatCount(tiers.team.starts)} starts on ${usage.passesOn}`}
        actions={act(
          <Action tone="primary" size="sm">
            {`Add ${block} starts`}
          </Action>,
        )}
      >
        Jobs keep running past the limit. {block} more starts cost{" "}
        {monthly(addOnPrices.startsPrice)}.
      </Notice>
    );
  }
  if (freePassesOn) {
    return (
      <Notice
        tone="warning"
        title={`At this pace, ${workspace.name} passes the ${formatCount(tiers.free.starts)} starts of Free on ${freePassesOn}`}
        actions={act(
          <LinkAction to={plans} tone="primary" size="sm">
            Compare plans
          </LinkAction>,
        )}
      >
        Jobs keep running past it. Team includes {startsText(tiers.team.starts)}{" "}
        a month for {monthly(tiers.team.price)}.
      </Notice>
    );
  }
  if (scene === "free-scheduled" && usage.ends) {
    return (
      <Notice
        tone="info"
        title={`${tiers[usage.tier].name} ends on ${usage.ends}`}
        actions={act(
          <Action size="sm" icon={ExternalLink}>
            {`Keep ${tiers[usage.tier].name}`}
          </Action>,
        )}
      >
        <span className="nowrap">{workspace.name}</span> moves to Free on{" "}
        {freeFrom}. Jobs keep running on all {usage.machines} machines.
      </Notice>
    );
  }
  return undefined;
}
