// The notices of waiting labels from the facts of the API: a label that no
// place of its order fits, a label whose order holds no provider, a label
// whose Boat size Boat refused while other labels still start on Boat, a
// label whose order holds only Boat while overflow is off, and a label that
// runs on the largest size while Boat, which would run it, is at its limit or
// refuses new sandboxes.
// A fit notice names the full request with "fits" and never names the
// resource that is short, because the API does not say which one is. A
// refused size holds only its own label, so the full queue still promises
// Boat to the other labels.

import type { ReactNode } from "react";

import { LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths } from "../../components/paths.ts";
import { AskAdmin } from "../../components/role-actions.tsx";
import { unbroken } from "../../fixtures/format.ts";
import type { Role } from "../../fixtures/members.ts";
import { providersOnlyLabel } from "../../fixtures/order-scenes.ts";
import { overflow } from "../../fixtures/overflow.ts";
import {
  boatAtLimit,
  boatRefusing,
  overflowOff,
} from "../../fixtures/placement.ts";
import { boatOnlyLabel, nothingFitsLabel } from "../../fixtures/settings.ts";
import type { Size } from "../../fixtures/sizes.ts";
import { largestBoat } from "../../fixtures/wait-facts.ts";
import { HeldNotice } from "./held-notice.tsx";
import type { LiveScene, LiveState } from "./live-scenes.ts";

/** "32 vCPU, 128 GB of memory, and 200 GB of free disk": the full request. */
export function requestPhrase(size: Size): string {
  return `${unbroken(`${size.vcpu} vCPU`)}, ${unbroken(`${size.memoryGb} GB`)} of memory, and ${unbroken(`${size.freeDiskGb} GB`)} of free disk`;
}

function jobsWord(count: number): string {
  return count === 1 ? "1 job" : `${count} jobs`;
}

export function placeFactNotice(
  state: LiveState,
  scene: LiveScene,
  role: Role,
): ReactNode {
  const editLabel =
    role === "admin" ? (
      <LinkAction to={paths.nothingFits} size="sm">
        Edit label
      </LinkAction>
    ) : (
      <AskAdmin />
    );
  const waitTitle = `${jobsWord(scene.waiting)} with the label ${nothingFitsLabel.name} wait`;
  switch (state) {
    case "nothing-fits-request":
      return (
        <Notice tone="warning" title={waitTitle} actions={editLabel}>
          No machine of yours and no Boat size fits a job of{" "}
          {requestPhrase(nothingFitsLabel)}. Change the label, or let it run on
          the largest size.
        </Notice>
      );
    case "machines-only":
      return (
        <Notice tone="warning" title={waitTitle} actions={editLabel}>
          No machine of yours fits a job of {requestPhrase(nothingFitsLabel)},
          and the label runs on your machines only. Change the label.
        </Notice>
      );
    case "size-refused": {
      const refused = scene.waitingJobs.filter(
        (job) => job.label === boatOnlyLabel.name,
      ).length;
      return (
        <>
          <Notice
            tone="warning"
            title={`Boat refused the size of the label ${boatOnlyLabel.name}`}
          >
            Its jobs wait, and ciproxy tries Boat again.
          </Notice>
          <Notice tone="info" title="Your machines are full">
            {jobsWord(scene.waiting - refused)} wait for a free slot. Each one
            starts on Boat when it has waited {overflow.waitSeconds} seconds.
          </Notice>
        </>
      );
    }
    case "providers-only-off":
      return (
        <HeldNotice
          label={providersOnlyLabel}
          context={overflowOff}
          count={scene.waiting}
          role={role}
        />
      );
    case "largest-held":
      return (
        <HeldNotice
          label={largestBoat.label}
          context={boatAtLimit}
          count={scene.waiting}
          role={role}
        />
      );
    case "largest-refused":
      return (
        <HeldNotice
          label={largestBoat.label}
          context={boatRefusing}
          count={scene.waiting}
          role={role}
        />
      );
    default:
      return null;
  }
}
