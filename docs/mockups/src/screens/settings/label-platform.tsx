// The platform of a label in the label dialog, and what its jobs do when no
// place of its order fits them. The add dialog offers Linux and macOS, and
// says that the choice cannot change later. The edit dialog shows the
// platform with a lock, as it shows the name. A macOS label runs on the Macs
// only, because a cloud Mac bills 24 hours at least, so its dialog has no
// choice of "When nothing fits": its jobs wait for a free Mac. The largest
// size names no provider: it is the largest size of the first provider of
// the order that can take jobs and whose largest size holds the image. An
// order with no provider says to add one first.

import { RadioCard, RadioCardGroup } from "@firna/ui/radio";
import { SegmentedControl } from "@firna/ui/segmented";

import { noop } from "../../components/actions.tsx";
import { TextField } from "../../components/fields.tsx";
import { providersOf } from "../../fixtures/label-order.ts";
import { platformName, type Platform } from "../../fixtures/platform.ts";
import type { Label } from "../../fixtures/settings.ts";

const platformOptions = (["linux", "macos"] as const).map((value) => ({
  value,
  label: platformName[value],
}));

export function PlatformField({
  platform,
  edits,
}: {
  platform: Platform;
  /** The dialog edits a label, so the platform shows with a lock. */
  edits: boolean;
}) {
  if (edits) {
    return (
      <TextField
        label="Platform"
        value={platformName[platform]}
        hint="A label keeps its platform. To change it, add a new label."
        readOnly
      />
    );
  }
  return (
    <SegmentedControl
      label="Platform"
      options={platformOptions}
      value={platform}
      onChange={noop}
      hint="The platform of the VM of each job. It cannot change later."
    />
  );
}

/** What the jobs of a Linux label do when no place of its order fits. */
export function WhenNothingFitsField({ label }: { label: Label }) {
  if (label.platform === "macos") return null;
  return (
    <RadioCardGroup label="When nothing fits">
      <RadioCard
        title="Wait"
        body="Jobs stay queued until a place of the order fits them."
        checked={label.whenNothingFits === "wait"}
        onPress={noop}
      />
      <RadioCard
        title="Run on the largest size"
        body={
          providersOf(label.order).length > 0
            ? "Jobs run on the largest size of the first provider of the order that can take jobs."
            : "Add a provider to the order to use this choice."
        }
        checked={label.whenNothingFits === "largest-size"}
        onPress={noop}
      />
    </RadioCardGroup>
  );
}
