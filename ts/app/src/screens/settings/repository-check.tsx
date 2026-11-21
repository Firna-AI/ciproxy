// One private repository in the list of selected repositories: a check box
// with the full name of the repository as its accessible name. It is the
// Firna button in the checkbox role, so it keeps the shared focus glow.

import { StyleSheet, Text, View } from "react-native";
import { Button } from "@firna/ui/button";

import { colors, radii } from "@ciproxy/tokens";

import { CheckIcon } from "../../components/icons.tsx";
import { text } from "../../components/text.ts";

export function RepositoryCheck({
  name,
  checked,
  first,
  disabled,
  onToggle,
}: {
  name: string;
  checked: boolean;
  /** The first row has no hairline above it. */
  first: boolean;
  disabled: boolean;
  onToggle: () => void;
}) {
  return (
    <View style={[styles.row, first && styles.first]}>
      <Button
        role="checkbox"
        checked={checked}
        accessibilityLabel={name}
        tone="plain"
        disabled={disabled}
        onPress={onToggle}
        style={styles.button}
        content={
          <View style={styles.content}>
            <View style={[styles.box, checked && styles.boxChecked]}>
              {checked ? (
                <CheckIcon size={12} strokeWidth={3} color={colors.onSolid} />
              ) : null}
            </View>
            <Text style={[text.small, styles.name]}>{name}</Text>
          </View>
        }
      />
    </View>
  );
}

const styles = StyleSheet.create({
  row: { borderTopWidth: 1, borderTopColor: colors.border },
  first: { borderTopWidth: 0 },
  button: {
    height: "auto",
    justifyContent: "flex-start",
    paddingVertical: 8,
    paddingHorizontal: 12,
    borderRadius: 0,
  },
  content: { flexDirection: "row", alignItems: "center", gap: 10 },
  box: {
    width: 16,
    height: 16,
    alignItems: "center",
    justifyContent: "center",
    borderWidth: 1,
    borderColor: colors.border2,
    borderRadius: radii.sm,
    backgroundColor: colors.surface,
  },
  boxChecked: { borderColor: colors.primary, backgroundColor: colors.primary },
  name: { flexShrink: 1, color: colors.ink },
});
