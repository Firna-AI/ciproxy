// The dialog that edits a label or adds one: its name, its size, where its
// jobs may run, and what they do when nothing fits. Before the save, it says
// how many jobs of the size one machine runs, or that no machine fits it, or
// that no machine and no Boat size fits it.

import { RadioCard, RadioCardGroup } from "@firna/ui/radio";
import { Switch } from "@firna/ui/switch";

import { LinkAction, noop } from "../../components/actions.tsx";
import { Dialog } from "../../components/dialog.tsx";
import { Notice } from "../../components/feedback.tsx";
import { TextField } from "../../components/fields.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { ax42, slotSize } from "../../fixtures/machines.ts";
import {
  boatOnlyLabel,
  fitsOwnMachine,
  jobsPerMachine,
  labelNamed,
  nothingFitsLabel,
  type Label,
} from "../../fixtures/settings.ts";
import { fitsBoat, largestBoatSize, sizePhrase } from "../../fixtures/sizes.ts";

export type LabelDialogMode = "edit" | "no-machine-fits" | "nothing-fits";

type DialogCopy = {
  label: Label;
  title: string;
  save: string;
  /** The list behind the dialog, where Cancel and the save return. */
  list: CataloguePath;
};

const edited = labelNamed("ciproxy-8vcpu");

const dialogs: Record<LabelDialogMode, DialogCopy> = {
  edit: {
    label: edited,
    title: `Edit ${edited.name}`,
    save: "Save label",
    list: paths.settings,
  },
  "no-machine-fits": {
    label: boatOnlyLabel,
    title: "Add a label",
    save: "Add label",
    list: paths.settings,
  },
  "nothing-fits": {
    label: nothingFitsLabel,
    title: `Edit ${nothingFitsLabel.name}`,
    save: "Save label",
    list: paths.labelWaits,
  },
};

/** The part of one own machine that jobs can use. */
const machineOffer = sizePhrase({
  vcpu: ax42.slotsTotal * slotSize.vcpu,
  memoryGb: ax42.slotsTotal * slotSize.memoryGb,
});

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

function FitNote({ label }: { label: Label }) {
  if (fitsOwnMachine(label)) {
    const jobs = jobsPerMachine(label);
    return (
      <p className="caption">
        Each of your machines runs {jobs} {jobs === 1 ? "job" : "jobs"} of this
        size at a time.
      </p>
    );
  }
  if (fitsBoat(label)) {
    return (
      <Notice tone="warning" title={`No machine fits ${sizePhrase(label)}`}>
        Jobs with this label run on Boat. Your machines offer {machineOffer}{" "}
        each.
      </Notice>
    );
  }
  return (
    <Notice
      tone="warning"
      title={`No machine and no Boat size fits ${sizePhrase(label)}`}
    >
      Your machines offer {machineOffer} each, and the largest Boat size offers{" "}
      {sizePhrase(largestBoatSize)}.
    </Notice>
  );
}

export function LabelDialog({ mode }: { mode: LabelDialogMode }) {
  const { label, title, save, list } = dialogs[mode];
  return (
    <Dialog
      title={title}
      footer={
        <>
          <LinkAction to={list} tone="ghost">
            Cancel
          </LinkAction>
          <LinkAction to={list} tone="primary">
            {save}
          </LinkAction>
        </>
      }
    >
      <TextField label="Label" value={label.name} />
      <div className="field-row">
        <TextField label="vCPU" value={String(label.vcpu)} />
        <TextField label="Memory, GB" value={String(label.memoryGb)} />
        <TextField label="Disk, GB" value={String(label.diskGb)} />
      </div>
      <FitNote label={label} />
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
          body={`Jobs run on ${sizePhrase(largestBoatSize)}, the largest size of your Boat account.`}
          checked={label.whenNothingFits === "largest-boat-size"}
          onPress={noop}
        />
      </RadioCardGroup>
    </Dialog>
  );
}
