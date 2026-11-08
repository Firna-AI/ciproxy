// The form of the label dialog. The size fields hold the text that the
// person types; a size is whole GB and whole vCPU of at least 1, as the API
// requires. The name goes to the API as typed, because GitHub decides which
// names work in runs-on, and the dialog shows the refusal of GitHub.

import type { ApiFailure } from "../api/client.ts";
import type { LabelInput, LabelView, NothingFits } from "../api/schema.ts";
import type { FailureCopy } from "./claim.ts";
import { protectedLabelName } from "./settings-labels.ts";

export type LabelDraft = {
  name: string;
  vcpus: string;
  memory: string;
  disk: string;
  allowMachines: boolean;
  allowBoat: boolean;
  whenNothingFits: NothingFits;
};

export type SizeField = "vcpus" | "memory" | "disk";

export type DraftCheck =
  | { ok: true; input: LabelInput }
  | { ok: false; errors: Partial<Record<SizeField, string>> };

/** The largest value of a 32-bit field of the API. */
const largest = 2_147_483_647;

export const sizeError = "Enter a whole number of 1 or more.";

function whole(text: string): number | null {
  if (!/^\d+$/.test(text.trim())) return null;
  const value = Number(text.trim());
  return value >= 1 && value <= largest ? value : null;
}

export function editDraft(label: LabelInput): LabelDraft {
  return {
    name: label.name,
    vcpus: String(label.vcpus),
    memory: String(label.memory_gb),
    disk: String(label.disk_gb),
    allowMachines: label.allow_machines,
    allowBoat: label.allow_boat,
    whenNothingFits: label.when_nothing_fits,
  };
}

/** A new label starts with the size of `ciproxy`, when the workspace has it. */
export function newDraft(labels: readonly LabelView[]): LabelDraft {
  const base = labels.find((view) => view.label.name === protectedLabelName);
  return {
    name: "",
    vcpus: base ? String(base.label.vcpus) : "",
    memory: base ? String(base.label.memory_gb) : "",
    disk: base ? String(base.label.disk_gb) : "",
    allowMachines: true,
    allowBoat: true,
    whenNothingFits: "wait",
  };
}

/** The label of the draft, or the size fields that need a fix. */
export function checkDraft(draft: LabelDraft): DraftCheck {
  const vcpus = whole(draft.vcpus);
  const memory = whole(draft.memory);
  const disk = whole(draft.disk);
  if (vcpus === null || memory === null || disk === null) {
    const errors: Partial<Record<SizeField, string>> = {};
    if (vcpus === null) errors.vcpus = sizeError;
    if (memory === null) errors.memory = sizeError;
    if (disk === null) errors.disk = sizeError;
    return { ok: false, errors };
  }
  return {
    ok: true,
    input: {
      name: draft.name,
      vcpus,
      memory_gb: memory,
      disk_gb: disk,
      allow_machines: draft.allowMachines,
      allow_boat: draft.allowBoat,
      when_nothing_fits: draft.whenNothingFits,
    },
  };
}

/** Whether a save can start: a name, and a size that the API accepts. */
export function canSave(draft: LabelDraft): boolean {
  return draft.name !== "" && checkDraft(draft).ok;
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

/**
 * The parts of a label that its machine fit depends on: the size and the
 * switch "Your machines". A change of another part needs no new fit.
 */
export function fitKey(input: LabelInput): string {
  return [
    input.vcpus,
    input.memory_gb,
    input.disk_gb,
    input.allow_machines,
  ].join("/");
}
