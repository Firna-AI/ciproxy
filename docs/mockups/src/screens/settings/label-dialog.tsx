// The dialog that edits a label or adds one: its name, its size, where its
// jobs may run, and what they do when nothing fits. The disk of the size is
// the free disk of each job, with its least value as the hint, and the line
// under it gives the VM disk with the current image, as the answer to item 77
// says. Before the save, it says how many jobs of the size one machine runs,
// or that no machine fits it, or that no machine and no Boat size fits it. A
// label name follows the GitHub label rules and never changes, as design call
// 33 says: the add dialog refuses a name that breaks the rules, and the edit
// dialog shows the name with a lock, beside Delete label, which a rename uses.
// A label keeps its platform too: the add dialog asks for it, and the edit
// dialog shows it with a lock. A macOS label runs on the Macs only.

import { Action, LinkAction } from "../../components/actions.tsx";
import { Dialog } from "../../components/dialog.tsx";
import { TextField } from "../../components/fields.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { unbroken } from "../../fixtures/format.ts";
import { macScene } from "../../fixtures/macs.ts";
import { machineScenes, type Machine } from "../../fixtures/machines.ts";
import {
  boatOnlyLabel,
  labelNamed,
  largerMacosLabel,
  macosLabel,
  nothingFitsLabel,
  refusedLabel,
  type Label,
} from "../../fixtures/settings.ts";
import { minimumFreeDiskGb } from "../../fixtures/sizes.ts";
import { diskText, vmDiskBytes } from "../../fixtures/vm-disk.ts";
import { FitNote } from "./label-fit.tsx";
import { LabelPlaces, PlatformField } from "./label-platform.tsx";

export type LabelDialogMode =
  | "edit"
  | "no-machine-fits"
  | "nothing-fits"
  | "mixed-machines"
  | "refused-name"
  | "no-boat"
  | "add-macos"
  | "edit-macos";

type DialogCopy = {
  label: Label;
  /** The dialog edits a label; otherwise it adds one. */
  edits: boolean;
  /** The list behind the dialog, where Cancel and the save return. */
  list: CataloguePath;
  /** The machines that decide how many jobs of the size fit. */
  machines: readonly Machine[];
  /** The refusal of the name, when the GitHub label rules refuse it. */
  nameError?: string;
  /** The workspace has no Boat account, so no Boat size is known. */
  noBoat?: boolean;
};

const steady = machineScenes.steady;

/** The draft of ciproxy-medium before the save: an Admin raises its free disk. */
const largerDiskDraft: Label = {
  ...labelNamed("ciproxy-medium"),
  freeDiskGb: 40,
};

const dialogs: Record<LabelDialogMode, DialogCopy> = {
  edit: {
    label: labelNamed("ciproxy-large"),
    edits: true,
    list: paths.settings,
    machines: steady,
  },
  "no-machine-fits": {
    label: boatOnlyLabel,
    edits: false,
    list: paths.settings,
    machines: steady,
  },
  "nothing-fits": {
    label: nothingFitsLabel,
    edits: true,
    list: paths.labelWaits,
    machines: steady,
  },
  "mixed-machines": {
    label: largerDiskDraft,
    edits: true,
    list: paths.settings,
    machines: machineScenes.fourMachines,
  },
  "refused-name": {
    label: refusedLabel,
    edits: false,
    list: paths.settings,
    machines: steady,
    nameError: `A label name cannot have spaces. Use ${refusedLabel.name.replaceAll(" ", "-")} instead.`,
  },
  "no-boat": {
    label: labelNamed("ciproxy-large"),
    edits: true,
    list: paths.labelsNoBoat,
    machines: steady,
    noBoat: true,
  },
  "add-macos": {
    label: largerMacosLabel,
    edits: false,
    list: paths.labelsMacos,
    machines: macScene,
  },
  "edit-macos": {
    label: macosLabel,
    edits: true,
    list: paths.labelsMacos,
    machines: macScene,
  },
};

function Footer({ copy }: { copy: DialogCopy }) {
  return (
    <>
      {copy.edits ? (
        <span className="dialog-footer-start">
          <Action tone="danger">Delete label</Action>
        </span>
      ) : null}
      <LinkAction to={copy.list} tone="ghost">
        Cancel
      </LinkAction>
      {copy.nameError ? (
        <Action tone="primary" disabled>
          Add label
        </Action>
      ) : (
        <LinkAction to={copy.list} tone="primary">
          {copy.edits ? "Save label" : "Add label"}
        </LinkAction>
      )}
    </>
  );
}

export function LabelDialog({ mode }: { mode: LabelDialogMode }) {
  const copy = dialogs[mode];
  const { label } = copy;
  return (
    <Dialog
      title={copy.edits ? `Edit ${label.name}` : "Add a label"}
      footer={<Footer copy={copy} />}
    >
      {copy.edits ? (
        <TextField
          label="Label"
          value={label.name}
          hint="A label keeps its name. To rename it, add a new label, then delete this one."
          readOnly
        />
      ) : (
        <TextField
          label="Label"
          value={label.name}
          hint={
            copy.nameError
              ? undefined
              : "The name that workflows use in runs-on. It cannot change later."
          }
          error={copy.nameError}
        />
      )}
      <PlatformField platform={label.platform} edits={copy.edits} />
      <div className="stack-tight">
        <div className="field-row">
          <TextField label="vCPU" value={String(label.vcpu)} />
          <TextField label="Memory, GB" value={String(label.memoryGb)} />
          <TextField
            label="Free disk, GB"
            value={String(label.freeDiskGb)}
            hint={`At least ${unbroken(`${minimumFreeDiskGb} GB`)}`}
          />
        </div>
        <p className="caption">
          With the current image, each VM has a disk of{" "}
          {diskText(vmDiskBytes(label))}.
        </p>
      </div>
      <FitNote
        label={label}
        machines={copy.machines}
        boat={copy.noBoat !== true}
      />
      <LabelPlaces label={label} />
    </Dialog>
  );
}
