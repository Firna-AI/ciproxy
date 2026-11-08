// The notices of the live queue whose wait reason comes from current facts of
// the API, and the current-contract notices of a suspended App and a Boat
// refusal. Each notice names only what the API gives: no person who suspended
// the App, no date of a suspension, no reason of a Boat refusal, and no Boat
// account address. A full queue promises Boat only to the jobs of a label
// that Boat can start now, at once when no own machine runs the label, and a
// Boat hold promises a free own slot only to the jobs whose label fits the
// own machines. A label that starts on the largest Boat size names its full
// request, and never the resource that is short. A setting that only an Admin
// changes gets its action for an Admin and Ask an Admin for everyone else.

import type { ReactNode } from "react";
import { ExternalLink } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { AskAdmin } from "../../components/role-actions.tsx";
import { formatMoney } from "../../fixtures/format.ts";
import { boat } from "../../fixtures/machines.ts";
import type { Role } from "../../fixtures/members.ts";
import { labsOrg, org } from "../../fixtures/org.ts";
import { boatOnlyLabel } from "../../fixtures/settings.ts";
import { boatSizeNamed, sizePhrase } from "../../fixtures/sizes.ts";
import { boatRefused } from "../../fixtures/waiting-jobs.ts";
import { largestBoat, unknownFit } from "../../fixtures/wait-facts.ts";
import type { LiveScene, LiveState } from "./live-scenes.ts";
import { placeFactNotice, requestPhrase } from "./place-fact-notices.tsx";

function adminLink(role: Role, to: CataloguePath, label: string): ReactNode {
  if (role !== "admin") return <AskAdmin />;
  return (
    <LinkAction to={to} size="sm">
      {label}
    </LinkAction>
  );
}

function jobsWord(count: number): string {
  return count === 1 ? "1 job" : `${count} jobs`;
}

export function factNotice(
  state: LiveState,
  scene: LiveScene,
  role: Role,
): ReactNode {
  const largest = boatSizeNamed(largestBoat.boatSize);
  switch (state) {
    case "unknown-fit":
      return (
        <Notice
          tone="info"
          title={`${jobsWord(scene.waiting)} with the label ${unknownFit.label.name} wait`}
        >
          Some machine or Boat facts are not available now, so ciproxy cannot
          tell where this label fits. The jobs start as soon as a machine or
          Boat can run them.
        </Notice>
      );
    case "largest-boat":
      return (
        <Notice
          tone="info"
          title={`Jobs with the label ${largestBoat.label.name} start on the largest Boat size`}
          actions={adminLink(role, paths.nothingFits, "Edit label")}
        >
          No machine of yours and no Boat size fits a job of{" "}
          {requestPhrase(largestBoat.label)}, so ciproxy starts them on Boat{" "}
          {largest.name}, with {sizePhrase(largest)}.
        </Notice>
      );
    case "boat-key":
      return (
        <Notice
          tone="warning"
          title="Boat rejected the key"
          actions={adminLink(
            role,
            paths.overflowBoatRejected,
            "Connect a new key",
          )}
        >
          Jobs run on your machines only, and wait there for a free slot, until
          a new key is connected.
        </Notice>
      );
    case "boat-billing":
      return (
        <Notice tone="warning" title="Boat needs billing on your Boat account">
          Boat starts no new sandboxes until its billing is set up. Jobs run on
          your machines only, and wait there for a free slot.
        </Notice>
      );
    case "overflow-off":
      return (
        <Notice
          tone="info"
          title="Your machines are full"
          actions={adminLink(role, paths.overflow, "Turn on Boat")}
        >
          {jobsWord(scene.waiting)} wait for a free slot. Boat overflow is off,
          so each one starts when a slot is free.
        </Notice>
      );
    case "boat-not-connected":
      return (
        <Notice
          tone="info"
          title="Your machines are full"
          actions={adminLink(role, paths.overflowNotConnected, "Connect Boat")}
        >
          {jobsWord(scene.waiting)} wait for a free slot. Boat is not connected,
          so each one starts when a slot is free.
        </Notice>
      );
    case "boat-at-once": {
      const atOnce = scene.waitingJobs.filter(
        (job) => job.label === boatOnlyLabel.name,
      ).length;
      return (
        <Notice tone="info" title="Your machines are full">
          {jobsWord(scene.waiting - atOnce)} wait for a free slot. Each one
          starts on Boat when it has waited {boat.waitSeconds} seconds.{" "}
          {jobsWord(atOnce)} start on Boat at once.
        </Notice>
      );
    }
    case "spend-uncounted":
      return (
        <Notice tone="info" title="Your machines are full">
          {jobsWord(scene.waiting)} wait for a free slot. Boat starts no new
          sandbox until ciproxy can count its costs.
        </Notice>
      );
    case "limit-boat-only":
      return (
        <Notice
          tone="warning"
          title={`Boat reached its ${formatMoney(boat.spendLimit)} limit for November`}
          actions={adminLink(
            role,
            paths.overflowLimitReached,
            "Raise the limit",
          )}
        >
          Jobs whose label fits your machines wait there for a free slot. Boat
          takes jobs again on {boat.nextMonth}.
        </Notice>
      );
    case "app-removed":
      return (
        <Notice
          tone="warning"
          title={`The GitHub App was removed from ${labsOrg.login}`}
          actions={
            <Action size="sm" icon={ExternalLink}>
              Install the App
            </Action>
          }
        >
          Jobs of <span className="nowrap">{labsOrg.login}</span> no longer run
          on ciproxy. An owner can install the App again to keep its job
          history.
        </Notice>
      );
    case "suspended-current":
      return (
        <Notice
          tone="warning"
          title={`The GitHub App is suspended on ${org.login}`}
          actions={
            <Action size="sm" icon={ExternalLink}>
              Open GitHub
            </Action>
          }
        >
          New jobs of <span className="nowrap">{org.login}</span> wait on GitHub
          until an owner unsuspends it, and running jobs finish.
        </Notice>
      );
    case "refused-current":
      return (
        <Notice tone="warning" title="Boat refused new sandboxes">
          Waiting jobs take the first free slot on your machines, and ciproxy
          tries Boat again at {boatRefused.retryAt}.
        </Notice>
      );
    default:
      return placeFactNotice(state, scene, role);
  }
}
