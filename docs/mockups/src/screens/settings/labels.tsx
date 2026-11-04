// Labels and their sizes. A workflow picks a label in runs-on; each label
// starts a VM of its size on an own machine, or on Boat. The dialog states
// open over the list: edit ciproxy-8vcpu, add a label that fits no machine,
// and edit ciproxy-32vcpu, which nothing fits. The list is the moment of 18
// November, when a request for access waits on the Members tab.

import { Plus } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { ReadFailure } from "../../components/feedback.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { unbroken } from "../../fixtures/format.ts";
import { openRequests } from "../../fixtures/members.ts";
import { labels, nothingFitsLabel } from "../../fixtures/settings.ts";
import { LabelDialog, type LabelDialogMode } from "./label-dialog.tsx";
import { LabelList } from "./label-list.tsx";
import { SettingsFrame } from "./settings-frame.tsx";

export const labelsStates = [
  "list",
  "edit-label",
  "no-machine-fits",
  "nothing-fits",
  "label-waits",
  "loading",
  "error",
] as const;
export type LabelsState = (typeof labelsStates)[number];

const dialogModes: Partial<Record<LabelsState, LabelDialogMode>> = {
  "edit-label": "edit",
  "no-machine-fits": "no-machine-fits",
  "nothing-fits": "nothing-fits",
};

/** The list holds ciproxy-32vcpu once mokly-ai added it for load tests. */
function labelsOf(state: LabelsState) {
  return state === "nothing-fits" || state === "label-waits"
    ? [...labels, nothingFitsLabel]
    : labels;
}

export function LabelsScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: LabelsState;
}) {
  const mode = dialogModes[state];
  return (
    <SettingsFrame
      viewport={viewport}
      active={paths.settings}
      openRequests={state === "list" ? openRequests.length : 0}
      overlay={mode ? <LabelDialog mode={mode} /> : undefined}
    >
      {state === "error" ? (
        <Panel>
          <ReadFailure subject="settings" retry={paths.settings} />
        </Panel>
      ) : (
        <>
          <p className="small muted settings-intro">
            A workflow picks a label in{" "}
            <span className="inline-code">runs-on</span>. Each label starts a
            fresh VM of its size, on your machines first and on Boat when they
            are busy.
          </p>
          <Panel
            title="Labels"
            flush
            action={
              <LinkAction to={paths.noMachineFits} size="sm" icon={Plus}>
                Add label
              </LinkAction>
            }
          >
            <LabelList
              viewport={viewport}
              labels={labelsOf(state)}
              loading={state === "loading"}
            />
          </Panel>
          <p className="caption">
            A slot on your machines is {unbroken("2 vCPU")} and{" "}
            {unbroken("8 GB")}. A larger size takes several slots.
          </p>
        </>
      )}
    </SettingsFrame>
  );
}
