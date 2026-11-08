// A quantity that a payer sets, such as the blocks of extra starts: the label,
// a caption, and Fewer and More buttons around the number. A person whose
// role cannot change it reads the number alone.

import { StyleSheet, Text, View } from "react-native";
import type { IconComponentProps } from "@firna/ui";
import { Button } from "@firna/ui/button";

import { PlusIcon } from "../icons.tsx";
import { text } from "../text.ts";

/** The minus stroke of the Fewer button, on the grid of the app icons. */
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
      style={{ flexShrink: 0 }}
    >
      <path d="M5 12h14" />
    </svg>
  );
}

export function Quantity({
  label,
  caption,
  value,
  onChange,
  disabled = false,
}: {
  label: string;
  caption: string;
  value: number;
  /** Absent for a person who cannot change the quantity. */
  onChange?: (value: number) => void;
  disabled?: boolean;
}) {
  return (
    <View style={styles.row}>
      <View style={styles.text}>
        <Text style={[text.body, text.strong]}>{label}</Text>
        <Text style={text.caption}>{caption}</Text>
      </View>
      {onChange === undefined ? (
        <Text style={[text.body, styles.value]}>{String(value)}</Text>
      ) : (
        <View style={styles.control}>
          <Button
            tone="secondary"
            size="sm"
            shape="square"
            icon={MinusIcon}
            accessibilityLabel={`Fewer: ${label}`}
            disabled={disabled || value === 0}
            onPress={() => onChange(value - 1)}
          />
          <Text
            style={[text.body, styles.value]}
            aria-live="polite"
            accessibilityLabel={`${label}: ${value}`}
          >
            {String(value)}
          </Text>
          <Button
            tone="secondary"
            size="sm"
            shape="square"
            icon={PlusIcon}
            accessibilityLabel={`More: ${label}`}
            disabled={disabled}
            onPress={() => onChange(value + 1)}
          />
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
  },
  text: { flexGrow: 1, flexShrink: 1, flexBasis: 160, gap: 2 },
  control: { flexDirection: "row", alignItems: "center", gap: 10 },
  value: {
    minWidth: 24,
    textAlign: "center",
    fontVariant: ["tabular-nums"],
  },
});
