// Labels and their sizes. A workflow picks a label in runs-on; each label
// starts a VM of its size at the first place of its order that can take the
// job. The dialog states open over the list: edit ciproxy-large, add a label
// that fits no machine, edit ciproxy-32vcpu, which nothing fits, raise the
// free disk of ciproxy-medium while the machines fit different numbers of its
// jobs, and add a label whose name GitHub refuses. The list is the moment of
// 18 November, when a request for access waits on the Members tab. What-ifs
// show a workspace with no provider, whose labels run on the machines only, a
// Mac with the macOS labels, an empty order, Boat before the own machines,
// two providers, Boat and Second provider, and the same two while the
// deployment has no settings for Boat.

import { Plus } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action, LinkAction } from "../../components/actions.tsx";
import { ReadFailure } from "../../components/feedback.tsx";
import { Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { usageBanner } from "../../components/usage-banner.tsx";
import { slotSize } from "../../fixtures/hardware.ts";
import { openRequests } from "../../fixtures/members.ts";
import { macosVmLimit } from "../../fixtures/platform.ts";
import { sizePhrase } from "../../fixtures/sizes.ts";
import { LabelDialog } from "./label-dialog.tsx";
import { LabelList } from "./label-list.tsx";
import {
  addLabelPath,
  contextOf,
  dialogModes,
  editsOf,
  labelsOf,
  withMacs,
  type LabelsState,
} from "./labels-scenes.ts";
import { SettingsFrame } from "./settings-frame.tsx";

export function LabelsScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: LabelsState;
}) {
  const mode = dialogModes[state];
  const addPath = addLabelPath(state);
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
            fresh VM of its platform and size at the first place of its order
            that can take the job. Jobs of a macOS label run on your Macs only.
          </p>
          <Panel
            title="Labels"
            flush
            action={
              addPath ? (
                <LinkAction to={addPath} size="sm" icon={Plus}>
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
              context={contextOf(state)}
              edits={editsOf(state)}
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
