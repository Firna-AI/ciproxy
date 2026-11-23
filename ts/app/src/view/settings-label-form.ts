// The form of the label dialog. The size fields hold the text that the
// person types; a size is whole GB and whole vCPU of at least 1, as the API
// requires. The name goes to the API as typed, because GitHub decides which
// names work in runs-on, and the dialog shows the refusal of GitHub. A label
// keeps its platform, as it keeps its name. A macOS label runs on the Macs
// only, so it always goes to the API with an order of own machines and
// "Wait". A new label starts with the order that the API gives a new label:
// own machines, then each offered provider. Until the person changes that
// order, Add label leaves it to the API. The free disk of a label can have a
// most value, such as the free space of the macOS image, when the API gives
// one.

import type { ApiFailure } from "../api/client.ts";
import type {
  LabelCreate,
  LabelInput,
  LabelPlace,
  LabelView,
  NothingFits,
  OverflowProvider,
  Platform,
} from "../api/schema.ts";
import type { FailureCopy } from "./claim.ts";
import { labelPlatform } from "./platform.ts";
import { defaultOrder, machinePlace } from "./settings-label-places.ts";
import { protectedLabelName } from "./settings-labels.ts";

export type LabelDraft = {
  name: string;
  platform: Platform;
  vcpus: string;
  memory: string;
  disk: string;
  /** The places of a Linux label, in order. */
  order: LabelPlace[];
  /** A new label keeps the order that the API gives a new label. */
  defaultOrder: boolean;
  whenNothingFits: NothingFits;
};

export type SizeField = "vcpus" | "memory" | "disk";

export type DraftCheck =
  | { ok: true; input: LabelInput }
  | { ok: false; errors: Partial<Record<SizeField, string>> };

/** The largest value of a 32-bit field of the API. */
const largest = 2_147_483_647;

export const sizeError = "Enter a whole number of 1 or more.";

/** The most free disk of a label, when the API gives one, in whole GB. */
export type DraftLimits = { maxDiskGb: number | null };

export const noLimits: DraftLimits = { maxDiskGb: null };

export function diskLimitError(maxDiskGb: number): string {
  return `Enter ${maxDiskGb} GB or less.`;
}

function whole(text: string): number | null {
  if (!/^\d+$/.test(text.trim())) return null;
  const value = Number(text.trim());
  return value >= 1 && value <= largest ? value : null;
}

export function editDraft(label: LabelInput): LabelDraft {
  return {
    name: label.name,
    platform: labelPlatform(label),
    vcpus: String(label.vcpus),
    memory: String(label.memory_gb),
    disk: String(label.disk_gb),
    order: label.order,
    defaultOrder: false,
    whenNothingFits: label.when_nothing_fits,
  };
}

/**
 * A new label starts with the size of `ciproxy`, when the workspace has it,
 * and with own machines, then each offered provider.
 */
export function newDraft(
  labels: readonly LabelView[],
  offered: readonly OverflowProvider[],
): LabelDraft {
  const base = labels.find((view) => view.label.name === protectedLabelName);
  return {
    name: "",
    platform: "linux_x64",
    vcpus: base ? String(base.label.vcpus) : "",
    memory: base ? String(base.label.memory_gb) : "",
    disk: base ? String(base.label.disk_gb) : "",
    order: defaultOrder(offered),
    defaultOrder: true,
    whenNothingFits: "wait",
  };
}

/** The draft with a new order, which the person chose. */
export function withOrder(draft: LabelDraft, order: LabelPlace[]): LabelDraft {
  return { ...draft, order, defaultOrder: false };
}

/** The free disk, or its error: below 1 GB, or above the most free disk. */
function diskOf(
  text: string,
  limits: DraftLimits,
): { gb: number } | { error: string } {
  const gb = whole(text);
  if (gb === null) return { error: sizeError };
  if (limits.maxDiskGb !== null && gb > limits.maxDiskGb) {
    return { error: diskLimitError(limits.maxDiskGb) };
  }
  return { gb };
}

function draftOrder(draft: LabelDraft): LabelPlace[] {
  return draft.platform === "macos_arm64" ? [machinePlace] : draft.order;
}

/**
 * The request of Add label. A Linux label whose order the person did not
 * change leaves the order to the API, which gives own machines, then each
 * offered provider.
 */
export function createRequest(
  input: LabelInput,
  draft: Pick<LabelDraft, "defaultOrder" | "platform">,
): LabelCreate {
  if (draft.defaultOrder && draft.platform !== "macos_arm64") {
    return { ...input, order: undefined };
  }
  return input;
}

/** The label of the draft, or the size fields that need a fix. */
export function checkDraft(
  draft: LabelDraft,
  limits: DraftLimits = noLimits,
): DraftCheck {
  const vcpus = whole(draft.vcpus);
  const memory = whole(draft.memory);
  const disk = diskOf(draft.disk, limits);
  if (vcpus === null || memory === null || "error" in disk) {
    const errors: Partial<Record<SizeField, string>> = {};
    if (vcpus === null) errors.vcpus = sizeError;
    if (memory === null) errors.memory = sizeError;
    if ("error" in disk) errors.disk = disk.error;
    return { ok: false, errors };
  }
  const mac = draft.platform === "macos_arm64";
  return {
    ok: true,
    input: {
      name: draft.name,
      platform: draft.platform,
      vcpus,
      memory_gb: memory,
      disk_gb: disk.gb,
      order: draftOrder(draft),
      when_nothing_fits: mac ? "wait" : draft.whenNothingFits,
    },
  };
}

/** Whether a save can start: a name, and a size that the API accepts. */
export function canSave(
  draft: LabelDraft,
  limits: DraftLimits = noLimits,
): boolean {
  return draft.name !== "" && checkDraft(draft, limits).ok;
}

/** A refused save: the refusal of the name by GitHub, or a failure. */
export type LabelFailure =
  { kind: "name"; message: string } | { kind: "form"; copy: FailureCopy };

export function labelFailure(
  failure: ApiFailure,
  action: "save" | "delete",
): LabelFailure {
  if (
    failure.kind === "problem" &&
    failure.problem.code === "github_label_refused"
  ) {
    return { kind: "name", message: failure.problem.message };
  }
  const title =
    action === "save"
      ? "Unable to save the label"
      : "Unable to delete the label";
  if (failure.kind === "network") {
    return {
      kind: "form",
      copy: { title, body: "Check the connection, then try again." },
    };
  }
  const code = failure.kind === "problem" ? failure.problem.code : null;
  const body =
    code === "invalid_request"
      ? "Check the size, then try again."
      : code === "unavailable"
        ? "ciproxy could not reach GitHub or your machines. Try again."
        : "ciproxy could not finish the change. Try again.";
  return { kind: "form", copy: { title, body } };
}
