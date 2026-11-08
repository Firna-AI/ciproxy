// The slots of one machine, each with its job or its warm runner, and the
// warm runner count that an Admin sets with two buttons. Each press saves the
// new count at once; while it saves, the count shows the new value and the
// buttons wait. Other roles see the count without the buttons.

import type { IconComponentProps } from "@firna/ui";
import { Button } from "@firna/ui/button";
import { StyleSheet, Text, View } from "react-native";

import { colors } from "@ciproxy/tokens";

import type { MachineView } from "../../api/schema.ts";
import { Panel, TextLink } from "../../components/feature-parts.tsx";
import { PlusIcon } from "../../components/icons.tsx";
import { text } from "../../components/text.ts";
import { detailCopy } from "../../view/machines-copy.ts";
import type { SlotsModel } from "../../view/machines-detail.ts";
import { slotCaption } from "../../view/machines.ts";
import { markRadius, SlotStrip, slotFill } from "./machine-figures.tsx";

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

function SlotList({
  model,
}: {
  model: Extract<SlotsModel, { kind: "slots" }>;
}) {
  return (
    <View role="list">
      {model.rows.map((row, index) => (
        <View
          key={row.key}
          role="listitem"
          style={[styles.slotRow, index > 0 && styles.divider]}
        >
          <View
            style={[styles.slot, { backgroundColor: slotFill[row.state] }]}
          />
          <Text style={[text.caption, styles.slotNumber]}>{row.label}</Text>
          <Text
            style={[
              text.small,
              row.state === "job" ? styles.ink : styles.muted,
            ]}
          >
            {row.text}
          </Text>
        </View>
      ))}
    </View>
  );
}

export type WarmControl = {
  /** The count to show: the saved one, or the one that is saving. */
  value: number;
  saving: boolean;
  /** Null for a person who cannot change the count. */
  onChange: ((count: number) => void) | null;
};

function WarmRunners({ control }: { control: WarmControl }) {
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
          <Text style={[text.body, styles.value]} role="status">
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

export function SlotsPanel({
  machine,
  model,
  jobsHref,
  warm,
}: {
  machine: MachineView;
  model: SlotsModel;
  jobsHref: string;
  warm: WarmControl;
}) {
  return (
    <Panel
      title={detailCopy.slots}
      action={<TextLink href={jobsHref} label={detailCopy.jobHistory} />}
    >
      {model.kind === "message" ? (
        <Text style={[text.small, styles.muted]}>{model.text}</Text>
      ) : (
        <>
          <SlotStrip states={model.states} label={slotCaption(machine)} />
          <SlotList model={model} />
          <Text style={text.caption}>{model.caption}</Text>
          {model.fits ? <Text style={text.caption}>{model.fits}</Text> : null}
        </>
      )}
      <WarmRunners control={warm} />
    </Panel>
  );
}

const styles = StyleSheet.create({
  slotRow: {
    flexDirection: "row",
    alignItems: "center",
    gap: 10,
    paddingVertical: 8,
  },
  divider: { borderTopWidth: 1, borderTopColor: colors.border },
  slot: { width: 14, height: 6, borderRadius: markRadius },
  slotNumber: { minWidth: 48 },
  ink: { color: colors.ink, flexShrink: 1 },
  muted: { color: colors.muted, flexShrink: 1 },
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
