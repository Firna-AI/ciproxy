// The states of the Labels page: the list and the dialog of each state, the
// labels of the moment, the places that decide where their jobs run, and
// the labels whose dialog has a screen of its own. The what-ifs show a
// workspace with no provider, a Mac, an empty order, Boat before the own
// machines, and two providers, also while Boat is unavailable.

import { paths, type CataloguePath } from "../../components/paths.ts";
import { machinesOnly } from "../../fixtures/label-order.ts";
import { macScene } from "../../fixtures/macs.ts";
import {
  boatFirstLabel,
  emptyOrderLabel,
  twoProviderLabels,
} from "../../fixtures/order-scenes.ts";
import {
  boatReady,
  boatUnavailable,
  noProviderReady,
  twoReady,
  type PlaceContext,
} from "../../fixtures/placement.ts";
import {
  labels,
  macosLabel,
  macosLabels,
  nothingFitsLabel,
  type Label,
} from "../../fixtures/settings.ts";
import type { LabelDialogMode } from "./label-dialog-scenes.ts";
import type { EditPaths } from "./label-list.tsx";

export const labelsStates = [
  "list",
  "edit-label",
  "no-machine-fits",
  "nothing-fits",
  "label-waits",
  "mixed-machines",
  "refused-name",
  "no-provider",
  "no-provider-edit",
  "macos",
  "add-macos",
  "edit-macos",
  "empty-order",
  "boat-first",
  "two-providers",
  "two-providers-edit",
  "boat-unavailable",
  "loading",
  "error",
] as const;
export type LabelsState = (typeof labelsStates)[number];

export const dialogModes: Partial<Record<LabelsState, LabelDialogMode>> = {
  "edit-label": "edit",
  "no-machine-fits": "no-machine-fits",
  "nothing-fits": "nothing-fits",
  "mixed-machines": "mixed-machines",
  "refused-name": "refused-name",
  "no-provider-edit": "no-provider",
  "add-macos": "add-macos",
  "edit-macos": "edit-macos",
  "empty-order": "empty-order",
  "boat-first": "boat-first",
  "two-providers-edit": "two-providers",
  "boat-unavailable": "boat-unavailable",
};

/** The states of the what-if with mac-studio-1. */
export function withMacs(state: LabelsState): boolean {
  return state === "macos" || state === "add-macos" || state === "edit-macos";
}

function noProvider(state: LabelsState): boolean {
  return state === "no-provider" || state === "no-provider-edit";
}

function twoProviders(state: LabelsState): boolean {
  return (
    state === "two-providers" ||
    state === "two-providers-edit" ||
    state === "boat-unavailable"
  );
}

/** The labels with one of them in place of the label of the same name. */
function withLabel(label: Label): readonly Label[] {
  return labels.map((entry) => (entry.name === label.name ? label : entry));
}

/** The labels of each state, such as ciproxy-32vcpu for the load tests. */
export function labelsOf(state: LabelsState): readonly Label[] {
  if (withMacs(state)) return [...labels, ...macosLabels];
  if (noProvider(state)) {
    return labels.map((label) => ({ ...label, order: machinesOnly }));
  }
  if (twoProviders(state)) return twoProviderLabels;
  if (state === "empty-order") return withLabel(emptyOrderLabel);
  if (state === "boat-first") return withLabel(boatFirstLabel);
  return state === "nothing-fits" || state === "label-waits"
    ? [...labels, nothingFitsLabel]
    : labels;
}

export function contextOf(state: LabelsState): PlaceContext {
  if (withMacs(state)) return { ...boatReady, machines: macScene };
  if (noProvider(state)) return noProviderReady;
  if (state === "boat-unavailable") return boatUnavailable;
  return twoProviders(state) ? twoReady : boatReady;
}

/** The labels of each state whose dialog has a screen of its own. */
export function editsOf(state: LabelsState): EditPaths {
  if (noProvider(state)) return { "ciproxy-large": paths.editLabelNoProvider };
  if (twoProviders(state)) {
    return { "ciproxy-large": paths.editLabelTwoProviders };
  }
  return {
    "ciproxy-large": paths.editLabel,
    [nothingFitsLabel.name]: paths.nothingFits,
    [macosLabel.name]: paths.editMacosLabel,
  };
}

/** The add dialog that Add label opens, when the state has one. */
export function addLabelPath(state: LabelsState): CataloguePath | undefined {
  if (noProvider(state) || twoProviders(state)) return undefined;
  return withMacs(state) ? paths.addMacosLabel : paths.noMachineFits;
}
