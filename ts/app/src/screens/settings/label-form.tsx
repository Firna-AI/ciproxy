// The fields of the label dialog: the name, which an edit shows read-only
// because a label keeps its name, the size, the line that says where jobs
// of the size run, the switches "Your machines" and "Boat", and "When
// nothing fits". The size fields hold the typed text, and each shows its
// error once it has text or after a save.

import type { ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";
import { Input } from "@firna/ui/input";
import { RadioCard, RadioCardGroup } from "@firna/ui/radio";
import { Switch } from "@firna/ui/switch";

import { text } from "../../components/text.ts";
import type { LabelDraft, SizeField } from "../../view/settings-label-form.ts";

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

const sizeFields: readonly {
  field: SizeField;
  label: string;
  hint?: string;
}[] = [
  { field: "vcpus", label: "vCPU" },
  { field: "memory", label: "Memory, GB" },
  { field: "disk", label: "Free disk, GB", hint: "At least 1 GB" },
];

export function LabelForm({
  draft,
  edit,
  nameError,
  sizeErrors,
  fit,
  onChange,
}: {
  draft: LabelDraft;
  /** The dialog edits a saved label, whose name never changes. */
  edit: boolean;
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
      <View style={styles.sizes}>
        {sizeFields.map(({ field, label, hint }) => (
          <View key={field} style={styles.size}>
            <Input
              label={label}
              value={draft[field]}
              onChangeText={(value) => onChange({ ...draft, [field]: value })}
              inputMode="numeric"
              hint={sizeErrors[field] === undefined ? hint : undefined}
              error={sizeErrors[field] ?? null}
            />
          </View>
        ))}
      </View>
      {fit}
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
          body="Jobs run on the largest size of your Boat account, also when it is smaller than the label."
          checked={draft.whenNothingFits === "largest_boat"}
          onPress={() =>
            onChange({ ...draft, whenNothingFits: "largest_boat" })
          }
        />
      </RadioCardGroup>
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
