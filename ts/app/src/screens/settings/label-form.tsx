// The fields of the label dialog: the name and the platform, which an edit
// shows read-only because a label keeps both, the size, the line that says
// where jobs of the size run, the switches "Your machines" and "Boat", and
// "When nothing fits". A macOS label runs on the Macs only, so it has no
// switches and no "When nothing fits": its jobs wait for a free Mac. The
// size fields hold the typed text, and each shows its error once it has text
// or after a save.

import type { ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";
import { Input } from "@firna/ui/input";
import { RadioCard, RadioCardGroup } from "@firna/ui/radio";
import { SegmentedControl } from "@firna/ui/segmented";
import { Switch } from "@firna/ui/switch";

import type { Platform } from "../../api/schema.ts";
import { text } from "../../components/text.ts";
import { platformName } from "../../view/platform.ts";
import type { LabelDraft, SizeField } from "../../view/settings-label-form.ts";
import { largestBoatText } from "../../view/settings-label-sizes.ts";

function Toggle({
  label,
  caption,
  value,
  onChange,
}: {
  label: string;
  caption: string;
  value: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <View style={styles.toggle}>
      <View style={styles.toggleText}>
        <Text style={[text.small, text.strong]}>{label}</Text>
        <Text style={text.caption}>{caption}</Text>
      </View>
      <Switch
        value={value}
        onValueChange={onChange}
        accessibilityLabel={label}
      />
    </View>
  );
}

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

/** Where the jobs of a Linux label may run, and what they do then. */
function LinuxPlaces({
  draft,
  onChange,
}: {
  draft: LabelDraft;
  onChange: (next: LabelDraft) => void;
}) {
  return (
    <>
      <Toggle
        label="Your machines"
        caption="Jobs start here first"
        value={draft.allowMachines}
        onChange={(allowMachines) => onChange({ ...draft, allowMachines })}
      />
      <Toggle
        label="Boat"
        caption="When your machines are busy, or none of them fits"
        value={draft.allowBoat}
        onChange={(allowBoat) => onChange({ ...draft, allowBoat })}
      />
      <RadioCardGroup label="When nothing fits">
        <RadioCard
          title="Wait"
          body="Jobs stay queued until a machine or a Boat size fits."
          checked={draft.whenNothingFits === "wait"}
          onPress={() => onChange({ ...draft, whenNothingFits: "wait" })}
        />
        <RadioCard
          title="Run on the largest Boat size"
          body={largestBoatText}
          checked={draft.whenNothingFits === "largest_overflow"}
          onPress={() =>
            onChange({ ...draft, whenNothingFits: "largest_overflow" })
          }
        />
      </RadioCardGroup>
    </>
  );
}

export function LabelForm({
  draft,
  edit,
  maxDiskGb,
  nameError,
  sizeErrors,
  fit,
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
  onChange: (next: LabelDraft) => void;
}) {
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
      {draft.platform === "macos_arm64" ? (
        <Text style={text.small}>
          Jobs run on your Macs only, never on Boat. When no Mac is free, they
          wait.
        </Text>
      ) : (
        <LinuxPlaces draft={draft} onChange={onChange} />
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  form: { gap: 16 },
  sizes: { flexDirection: "row", gap: 12 },
  size: { flex: 1, minWidth: 0 },
  toggle: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 16,
  },
  toggleText: { flexShrink: 1, gap: 2 },
});
