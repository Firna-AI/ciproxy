// The fields of the label dialog: the name and the platform, which an edit
// shows read-only because a label keeps both, the size, the line that says
// where jobs of the size run, the order of places, and "When nothing fits".
// A macOS label runs on the Macs only, so its order holds own machines only
// and it has no "When nothing fits": its jobs wait for a free Mac. The size
// fields hold the typed text, and each shows its error once it has text or
// after a save.

import type { ReactNode } from "react";
import { StyleSheet, View } from "react-native";
import { Input } from "@firna/ui/input";
import { RadioCard, RadioCardGroup } from "@firna/ui/radio";
import { SegmentedControl } from "@firna/ui/segmented";

import type { OverflowProvider, Platform } from "../../api/schema.ts";
import { platformName } from "../../view/platform.ts";
import {
  withOrder,
  type LabelDraft,
  type SizeField,
} from "../../view/settings-label-form.ts";
import {
  machinePlace,
  orderProviders,
} from "../../view/settings-label-places.ts";
import { LabelOrder, type OrderFacts } from "./label-order.tsx";

const platformOptions = (["linux_x64", "macos_arm64"] as const).map(
  (value) => ({ value, label: platformName[value] }),
);

function PlatformField({
  draft,
  edit,
  onChange,
}: {
  draft: LabelDraft;
  edit: boolean;
  onChange: (next: LabelDraft) => void;
}) {
  if (edit) {
    return (
      <Input
        label="Platform"
        value={platformName[draft.platform]}
        editable={false}
        hint="A label keeps its platform. To change it, add a new label."
      />
    );
  }
  return (
    <SegmentedControl<Platform>
      label="Platform"
      options={platformOptions}
      value={draft.platform}
      onChange={(platform) => onChange({ ...draft, platform })}
      hint="The platform of the VM of each job. It cannot change later."
    />
  );
}

/** The free disk hint: its least value, and its most when there is one. */
function diskHint(maxDiskGb: number | null): string {
  return maxDiskGb === null ? "At least 1 GB" : `From 1 GB to ${maxDiskGb} GB`;
}

const sizeFields: readonly { field: SizeField; label: string }[] = [
  { field: "vcpus", label: "vCPU" },
  { field: "memory", label: "Memory, GB" },
  { field: "disk", label: "Free disk, GB" },
];

/** What the jobs of a Linux label do when no place of its order fits. */
function WhenNothingFits({
  draft,
  onChange,
}: {
  draft: LabelDraft;
  onChange: (next: LabelDraft) => void;
}) {
  return (
    <RadioCardGroup label="When nothing fits">
      <RadioCard
        title="Wait"
        body="Jobs stay queued until a place of the order fits them."
        checked={draft.whenNothingFits === "wait"}
        onPress={() => onChange({ ...draft, whenNothingFits: "wait" })}
      />
      <RadioCard
        title="Run on the largest size"
        body={
          orderProviders(draft.order).length > 0
            ? "Jobs run on the largest size of the first provider of the order that can take jobs."
            : "Add a provider to the order to use this choice."
        }
        checked={draft.whenNothingFits === "largest_overflow"}
        onPress={() =>
          onChange({ ...draft, whenNothingFits: "largest_overflow" })
        }
      />
    </RadioCardGroup>
  );
}

/** The facts of the order rows and the places that Add a place offers. */
export type OrderInput = OrderFacts & {
  /** The offered providers, or null while the overflow settings are unknown. */
  offered: readonly OverflowProvider[] | null;
  /** Settings under Overflow, where a provider connects. */
  overflowHref: string;
};

export function LabelForm({
  draft,
  edit,
  maxDiskGb,
  nameError,
  sizeErrors,
  fit,
  order,
  onChange,
}: {
  draft: LabelDraft;
  /** The dialog edits a saved label, whose name and platform never change. */
  edit: boolean;
  /** The most free disk of the label in GB, when the API gives one. */
  maxDiskGb: number | null;
  /** The refusal of the name by GitHub. */
  nameError: string | null;
  /** The size fields that need a fix and show it now. */
  sizeErrors: Partial<Record<SizeField, string>>;
  /** The line that says where jobs of the size run. */
  fit: ReactNode;
  order: OrderInput;
  onChange: (next: LabelDraft) => void;
}) {
  const mac = draft.platform === "macos_arm64";
  return (
    <View style={styles.form}>
      {edit ? (
        <Input
          label="Label"
          value={draft.name}
          editable={false}
          hint="A label keeps its name. To rename it, add a new label, then delete this one."
        />
      ) : (
        <Input
          label="Label"
          value={draft.name}
          onChangeText={(name) => onChange({ ...draft, name })}
          autoCapitalize="none"
          autoCorrect={false}
          spellCheck={false}
          hint={
            nameError === null
              ? "The name that workflows use in runs-on. It cannot change later."
              : undefined
          }
          error={nameError}
        />
      )}
      <PlatformField draft={draft} edit={edit} onChange={onChange} />
      <View style={styles.sizes}>
        {sizeFields.map(({ field, label }) => (
          <View key={field} style={styles.size}>
            <Input
              label={label}
              value={draft[field]}
              onChangeText={(value) => onChange({ ...draft, [field]: value })}
              inputMode="numeric"
              hint={
                sizeErrors[field] === undefined && field === "disk"
                  ? diskHint(maxDiskGb)
                  : undefined
              }
              error={sizeErrors[field] ?? null}
            />
          </View>
        ))}
      </View>
      {fit}
      <LabelOrder
        order={mac ? [machinePlace] : draft.order}
        mac={mac}
        facts={order}
        offered={order.offered}
        overflowHref={order.overflowHref}
        name={draft.name}
        onChange={(places) => onChange(withOrder(draft, places))}
      />
      {mac ? null : <WhenNothingFits draft={draft} onChange={onChange} />}
    </View>
  );
}

const styles = StyleSheet.create({
  form: { gap: 16 },
  sizes: { flexDirection: "row", gap: 12 },
  size: { flex: 1, minWidth: 0 },
});
