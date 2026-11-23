// The scenes of the label dialog: the label of each mode, whether it adds or
// edits it, the list behind it, and the places that decide where its jobs
// run. In the main story, Boat is the provider that mokly-ai connected. The
// what-ifs show a workspace with no provider, a fourth machine, the Macs, an
// empty order, Boat before the own machines, two providers, and two
// providers while the deployment has no settings for Boat.

import { paths, type CataloguePath } from "../../components/paths.ts";
import { machinesOnly } from "../../fixtures/label-order.ts";
import { macScene } from "../../fixtures/macs.ts";
import { machineScenes } from "../../fixtures/machines.ts";
import {
  boatFirstLabel,
  defaultSecondLast,
  emptyOrderLabel,
  largeSecondFirst,
} from "../../fixtures/order-scenes.ts";
import {
  boatReady,
  boatUnavailable,
  noProviderReady,
  twoReady,
  type PlaceContext,
} from "../../fixtures/placement.ts";
import type { ProviderKey } from "../../fixtures/providers.ts";
import {
  boatOnlyLabel,
  labelNamed,
  largerMacosLabel,
  macosLabel,
  nothingFitsLabel,
  refusedLabel,
  type Label,
} from "../../fixtures/settings.ts";

export type LabelDialogMode =
  | "edit"
  | "no-machine-fits"
  | "nothing-fits"
  | "mixed-machines"
  | "refused-name"
  | "no-provider"
  | "add-macos"
  | "edit-macos"
  | "empty-order"
  | "boat-first"
  | "two-providers"
  | "boat-unavailable";

export type DialogCopy = {
  label: Label;
  /** The dialog edits a label; otherwise it adds one. */
  edits: boolean;
  /** The list behind the dialog, where Cancel and the save return. */
  list: CataloguePath;
  /** The ready providers and the machines that decide where jobs run. */
  context: PlaceContext;
  /** The providers that the workspace connected once. */
  offered: readonly ProviderKey[];
  /** The refusal of the name, when the GitHub label rules refuse it. */
  nameError?: string;
};

/** The providers of the main story: Boat, connected in Setup. */
const boatOnly: readonly ProviderKey[] = ["boat"];

/** The Mac what-if: mac-studio-1 runs the jobs of the macOS labels. */
const macs: PlaceContext = { ...boatReady, machines: macScene };

/** The draft of ciproxy-medium before the save: an Admin raises its free disk. */
const largerDiskDraft: Label = {
  ...labelNamed("ciproxy-medium"),
  freeDiskGb: 40,
};

export const dialogs: Readonly<Record<LabelDialogMode, DialogCopy>> = {
  edit: {
    label: labelNamed("ciproxy-large"),
    edits: true,
    list: paths.settings,
    context: boatReady,
    offered: boatOnly,
  },
  "no-machine-fits": {
    label: boatOnlyLabel,
    edits: false,
    list: paths.settings,
    context: boatReady,
    offered: boatOnly,
  },
  "nothing-fits": {
    label: nothingFitsLabel,
    edits: true,
    list: paths.labelWaits,
    context: boatReady,
    offered: boatOnly,
  },
  "mixed-machines": {
    label: largerDiskDraft,
    edits: true,
    list: paths.settings,
    context: { ...boatReady, machines: machineScenes.fourMachines },
    offered: boatOnly,
  },
  "refused-name": {
    label: refusedLabel,
    edits: false,
    list: paths.settings,
    context: boatReady,
    offered: boatOnly,
    nameError: `A label name cannot have spaces. Use ${refusedLabel.name.replaceAll(" ", "-")} instead.`,
  },
  "no-provider": {
    label: { ...labelNamed("ciproxy-large"), order: machinesOnly },
    edits: true,
    list: paths.labelsNoProvider,
    context: noProviderReady,
    offered: [],
  },
  "add-macos": {
    label: largerMacosLabel,
    edits: false,
    list: paths.labelsMacos,
    context: macs,
    offered: boatOnly,
  },
  "edit-macos": {
    label: macosLabel,
    edits: true,
    list: paths.labelsMacos,
    context: macs,
    offered: boatOnly,
  },
  "empty-order": {
    label: emptyOrderLabel,
    edits: true,
    list: paths.settings,
    context: boatReady,
    offered: boatOnly,
  },
  "boat-first": {
    label: boatFirstLabel,
    edits: true,
    list: paths.settings,
    context: boatReady,
    offered: boatOnly,
  },
  "two-providers": {
    label: largeSecondFirst,
    edits: true,
    list: paths.labelsTwoProviders,
    context: twoReady,
    offered: ["boat", "second"],
  },
  "boat-unavailable": {
    label: defaultSecondLast,
    edits: true,
    list: paths.labelsTwoProviders,
    context: boatUnavailable,
    offered: ["boat", "second"],
  },
};
