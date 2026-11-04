// The notices above the live queue: why jobs wait. Every notice says what
// happens to the waiting jobs and offers one way forward. The way forward is
// a setting or the Boat account, which only an Admin changes, so a Viewer or
// a Billing person reads Ask an Admin in its place.

import type { ReactNode } from "react";
import { ExternalLink } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths } from "../../components/paths.ts";
import { AskAdmin } from "../../components/role-actions.tsx";
import { formatMoney } from "../../fixtures/format.ts";
import { boat } from "../../fixtures/machines.ts";
import type { Role } from "../../fixtures/members.ts";
import { nothingFitsLabel } from "../../fixtures/settings.ts";
import { sizePhrase } from "../../fixtures/sizes.ts";
import { boatRefused } from "../../fixtures/waiting-jobs.ts";
import type { LiveScene, LiveState } from "./live-scenes.ts";

export function liveNotice(
  state: LiveState,
  scene: LiveScene,
  role: Role,
): ReactNode {
  const admin = (action: ReactNode) =>
    role === "admin" ? action : <AskAdmin />;
  switch (state) {
    case "peak":
      return (
        <Notice tone="info" title="Your machines are full">
          {scene.waiting} jobs wait for a free slot. Each one starts on Boat
          when it has waited {boat.waitSeconds} seconds.
        </Notice>
      );
    case "spend-limit":
      return (
        <Notice
          tone="warning"
          title={`Boat reached its ${formatMoney(boat.spendLimit)} limit for November`}
          actions={admin(
            <LinkAction to={paths.overflowLimitReached} size="sm">
              Raise the limit
            </LinkAction>,
          )}
        >
          Jobs run on your machines only, and wait there for a free slot. Boat
          takes jobs again on {boat.nextMonth}.
        </Notice>
      );
    case "boat-refused":
      return (
        <Notice
          tone="warning"
          title="Boat refused new sandboxes"
          actions={admin(
            <Action size="sm" icon={ExternalLink}>
              Open Boat
            </Action>,
          )}
        >
          The start limit of your Boat plan is full until {boatRefused.retryAt}.
          Waiting jobs take the first free slot on your machines, and ciproxy
          tries Boat again then.
        </Notice>
      );
    case "nothing-fits":
      return (
        <Notice
          tone="warning"
          title={`${scene.waiting} jobs with the label ${nothingFitsLabel.name} wait`}
          actions={admin(
            <LinkAction to={paths.nothingFits} size="sm">
              Edit label
            </LinkAction>,
          )}
        >
          No machine and no Boat size has {sizePhrase(nothingFitsLabel)}. Change
          the label, or let it run on the largest Boat size.
        </Notice>
      );
    default:
      return null;
  }
}
