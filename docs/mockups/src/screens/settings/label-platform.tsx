// The platform of a label in the label dialog, and where its jobs may run. The
// add dialog offers Linux and macOS, and says that the choice cannot change
// later. The edit dialog shows the platform with a lock, as it shows the
// name. A Linux label runs on the Linux machines first and on Boat when they
// are busy. A macOS label runs on the Macs only, because a cloud Mac bills 24
// hours at least, so its dialog has no Boat switch and no choice of "When
// nothing fits": its jobs wait for a free Mac.

import { RadioCard, RadioCardGroup } from "@firna/ui/radio";
import { SegmentedControl } from "@firna/ui/segmented";
import { Switch } from "@firna/ui/switch";

import { noop } from "../../components/actions.tsx";
import { TextField } from "../../components/fields.tsx";
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

function Toggle({ label, caption }: { label: string; caption: string }) {
  return (
    <div className="toggle-row">
      <span className="stack-tight">
        <span className="strong">{label}</span>
        <span className="caption">{caption}</span>
      </span>
      <Switch value onValueChange={noop} accessibilityLabel={label} />
    </div>
  );
}

/** Where the jobs of the label may run, and what they do when nothing fits. */
export function LabelPlaces({ label }: { label: Label }) {
  if (label.platform === "macos") {
    return (
      <p className="small">
        Jobs run on your Macs only, never on Boat. When no Mac is free, they
        wait.
      </p>
    );
  }
  return (
    <>
      <Toggle label="Your machines" caption="Jobs start here first" />
      <Toggle
        label="Boat"
        caption="When your machines are busy, or none of them fits"
      />
      <RadioCardGroup label="When nothing fits">
        <RadioCard
          title="Wait"
          body="Jobs stay queued until a machine or a Boat size fits."
          checked={label.whenNothingFits === "wait"}
          onPress={noop}
        />
        <RadioCard
          title="Run on the largest Boat size"
          body="Jobs run on the largest size of your Boat account."
          checked={label.whenNothingFits === "largest-boat-size"}
          onPress={noop}
        />
      </RadioCardGroup>
    </>
  );
}
