// The warm runner count of the machine page: the saved count of the machine
// in every state, also while it drains, is offline, or has no slots. An Admin
// sets it with two buttons; each press saves the new count at once, and while
// it saves, the count shows the new value and the buttons wait. Other roles
// see the count without the buttons.

import type { IconComponentProps } from "@firna/ui";
import { Button } from "@firna/ui/button";
import { StyleSheet, Text, View } from "react-native";

import { colors } from "@ciproxy/tokens";

import { PlusIcon } from "../../components/icons.tsx";
import { text } from "../../components/text.ts";
import type { WarmControl } from "../../view/machines-controls.ts";
import { detailCopy } from "../../view/machines-copy.ts";

function MinusIcon({
  color,
  size = 16,
  strokeWidth = 1.75,
}: IconComponentProps) {
  return (
    <svg
      aria-hidden="true"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke={color ?? "currentColor"}
      strokeWidth={strokeWidth}
      strokeLinecap="round"
    >
      <path d="M5 12h14" />
    </svg>
  );
}

export function WarmRunners({ control }: { control: WarmControl }) {
  const { value, saving, onChange } = control;
  return (
    <View style={styles.quantity}>
      <View style={styles.quantityText}>
        <Text style={[text.body, text.strong]}>{detailCopy.warmRunners}</Text>
        <Text style={text.caption}>{detailCopy.warmCaption}</Text>
      </View>
      {onChange === null ? (
        <Text style={[text.body, styles.value]}>{value}</Text>
      ) : (
        <View style={styles.stepper}>
          <Button
            tone="secondary"
            size="sm"
            shape="square"
            icon={MinusIcon}
            accessibilityLabel={detailCopy.fewer}
            disabled={saving || value <= 0}
            onPress={
              saving || value <= 0 ? undefined : () => onChange(value - 1)
            }
          />
          <Text
            style={[text.body, styles.value]}
            role="status"
            aria-label={detailCopy.warmRunners}
          >
            {value}
          </Text>
          <Button
            tone="secondary"
            size="sm"
            shape="square"
            icon={PlusIcon}
            accessibilityLabel={detailCopy.more}
            disabled={saving}
            onPress={saving ? undefined : () => onChange(value + 1)}
          />
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  quantity: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
  quantityText: { flexGrow: 1, flexShrink: 1, flexBasis: 180, gap: 2 },
  stepper: { flexDirection: "row", alignItems: "center", gap: 10 },
  value: { minWidth: 20, textAlign: "center", fontVariant: ["tabular-nums"] },
});
