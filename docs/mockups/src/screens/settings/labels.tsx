// Labels and their sizes. A workflow picks a label in runs-on; each label
// starts a VM of its size on an own machine, or on Boat. The dialog states
// open over the list: edit ciproxy-large, add a label that fits no machine,
// edit ciproxy-32vcpu, which nothing fits, raise the free disk of
// ciproxy-medium while the machines fit different numbers of its jobs, and add
// a label whose name GitHub refuses. The list is the moment of 18 November,
// when a request for access waits on the Members tab. A what-if without a
// Boat account, after a setup that skipped Boat, shows the list and the edit
// of ciproxy-large, whose jobs run on the machines only. A what-if with a Mac
// shows the list with the macOS label ciproxy-macos, the add dialog with
// macOS picked, and the edit of ciproxy-macos.

import { Plus } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import { ReadFailure } from "../../components/feedback.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { usageBanner } from "../../components/usage-banner.tsx";
import { slotSize } from "../../fixtures/hardware.ts";
import { macScene } from "../../fixtures/macs.ts";
import { openRequests } from "../../fixtures/members.ts";
import { macosVmLimit } from "../../fixtures/platform.ts";
import {
  labels,
  macosLabels,
  nothingFitsLabel,
} from "../../fixtures/settings.ts";
import { sizePhrase } from "../../fixtures/sizes.ts";
import { LabelDialog, type LabelDialogMode } from "./label-dialog.tsx";
import { LabelList } from "./label-list.tsx";
import { SettingsFrame } from "./settings-frame.tsx";

export const labelsStates = [
  "list",
  "edit-label",
  "no-machine-fits",
  "nothing-fits",
  "label-waits",
  "mixed-machines",
  "refused-name",
  "no-boat",
  "no-boat-edit",
  "macos",
  "add-macos",
  "edit-macos",
  "loading",
  "error",
] as const;
export type LabelsState = (typeof labelsStates)[number];

const dialogModes: Partial<Record<LabelsState, LabelDialogMode>> = {
  "edit-label": "edit",
  "no-machine-fits": "no-machine-fits",
  "nothing-fits": "nothing-fits",
  "mixed-machines": "mixed-machines",
  "refused-name": "refused-name",
  "no-boat-edit": "no-boat",
  "add-macos": "add-macos",
  "edit-macos": "edit-macos",
};

/** The states of the what-if with mac-studio-1. */
function withMacs(state: LabelsState): boolean {
  return state === "macos" || state === "add-macos" || state === "edit-macos";
}

/** The list holds ciproxy-32vcpu once mokly-ai added it for load tests. */
function labelsOf(state: LabelsState) {
  if (withMacs(state)) return [...labels, ...macosLabels];
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
  const boat = state !== "no-boat" && state !== "no-boat-edit";
  return (
    <SettingsFrame
      viewport={viewport}
      active={paths.settings}
      openRequests={state === "list" ? openRequests.length : 0}
      banner={
        state === "mixed-machines" || withMacs(state)
          ? usageBanner("over-machines", "other")
          : undefined
      }
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
            fresh VM of its platform and size, on your machines first and on
            Boat when they are busy. Jobs of a macOS label run on your Macs
            only.
          </p>
          <Panel
            title="Labels"
            flush
            action={
              boat ? (
                <LinkAction
                  to={
                    withMacs(state) ? paths.addMacosLabel : paths.noMachineFits
                  }
                  size="sm"
                  icon={Plus}
                >
                  Add label
                </LinkAction>
              ) : (
                <Action size="sm" icon={Plus}>
                  Add label
                </Action>
              )
            }
          >
            <LabelList
              viewport={viewport}
              labels={labelsOf(state)}
              loading={state === "loading"}
              boat={boat}
              machines={withMacs(state) ? macScene : undefined}
            />
          </Panel>
          <p className="caption">
            A slot on your machines is {sizePhrase(slotSize)}. A larger size
            takes several slots. Each running job also holds disk for the image
            and its free disk.
            {withMacs(state)
              ? ` A Mac runs at most ${macosVmLimit} job VMs at a time.`
              : null}
          </p>
        </>
      )}
    </SettingsFrame>
  );
}
