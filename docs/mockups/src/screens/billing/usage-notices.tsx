// The notices of Plan and usage that only this page shows: the pace of the
// starts passes the tier before the period ends, or a paid tier ends after a
// move to Free. Limits are soft, so each notice says that jobs keep running
// and offers what covers the use. The banners of a passed limit or a failed
// payment show on every page (components/usage-banner.tsx).

import type { ReactNode } from "react";
import { ExternalLink } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths } from "../../components/paths.ts";
import { AskPayer } from "../../components/usage-banner.tsx";
import { formatCount, monthly } from "../../fixtures/format.ts";
import { org } from "../../fixtures/org.ts";
import { addOnPrices, startsText, tiers } from "../../fixtures/tiers.ts";
import { freeFrom, type Usage, type UsageScene } from "../../fixtures/usage.ts";

export function usageNotice(
  scene: UsageScene,
  usage: Usage,
  canPay: boolean,
): ReactNode {
  const block = formatCount(addOnPrices.startsBlock);
  const act = (actions: ReactNode) => (canPay ? actions : <AskPayer />);
  if (scene === "near-limit") {
    return (
      <Notice
        tone="warning"
        title={`At this pace, ${org.login} passes ${formatCount(tiers.team.starts)} starts on ${usage.passesOn}`}
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
  if (scene === "free" && usage.passesOn) {
    return (
      <Notice
        tone="warning"
        title={`At this pace, ${org.login} passes the ${formatCount(tiers.free.starts)} starts of Free on ${usage.passesOn}`}
        actions={act(
          <LinkAction to={paths.plans} tone="primary" size="sm">
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
        <span className="nowrap">{org.login}</span> moves to Free on {freeFrom}.
        Jobs keep running on all {usage.machines} machines.
      </Notice>
    );
  }
  return undefined;
}
