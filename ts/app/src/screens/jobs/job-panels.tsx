// The two panels of the job page. "Where it ran" names the place and why,
// with the times that GitHub gave on a timeline rail. "Details" lists the
// label, the VM size, the wait, the duration, the cache, and the start.

import { StyleSheet, Text, View } from "react-native";

import { colors, radii } from "@ciproxy/tokens";

import {
  DetailList,
  DetailRow,
  DetailStack,
} from "../../components/detail-list.tsx";
import {
  BoatIcon,
  DashedCircleIcon,
  MachinesIcon,
} from "../../components/icons.tsx";
import { Panel } from "../../components/panel.tsx";
import { text } from "../../components/text.ts";
import type { DetailRowModel, JobDetailModel } from "../../view/jobs-detail.ts";

function PlaceIcon({ model }: { model: JobDetailModel }) {
  const Icon =
    model.place.kind === "boat"
      ? BoatIcon
      : model.place.kind === "machine"
        ? MachinesIcon
        : DashedCircleIcon;
  return (
    <View style={styles.tile}>
      <Icon size={16} color={colors.ink2} />
    </View>
  );
}

export function PlacePanel({ model }: { model: JobDetailModel }) {
  return (
    <Panel title="Where it ran">
      <View style={styles.place}>
        <PlaceIcon model={model} />
        <View style={styles.placeText}>
          <Text
            style={[
              text.body,
              text.strong,
              model.place.kind === "machine" && text.mono,
            ]}
          >
            {model.placeTitle}
          </Text>
          <Text style={[text.small, styles.muted]}>{model.reason}</Text>
        </View>
      </View>
      <View style={styles.timeline} role="list" aria-label="Timeline">
        {model.timeline.map((event, index) => (
          <View
            key={`${event.time}-${index}`}
            style={styles.event}
            role="listitem"
          >
            <Text style={[text.small, text.mono, styles.time]}>
              {event.time}
            </Text>
            <Text style={[text.small, styles.eventText]}>
              {event.text}
              {event.machine ? (
                <Text style={[text.mono, styles.ink]}> {event.machine}</Text>
              ) : null}
            </Text>
          </View>
        ))}
      </View>
    </Panel>
  );
}

function RowValue({ row }: { row: DetailRowModel }) {
  // Each value is a text node of its own, so it takes the Graphite type in
  // place of the default type of the browser.
  const value = row.code ? (
    <Text style={[text.small, text.mono, styles.code]}>{row.value}</Text>
  ) : (
    <Text style={[text.body, styles.value]}>{row.value}</Text>
  );
  if (row.caption === null) return value;
  return <DetailStack caption={row.caption}>{value}</DetailStack>;
}

export function DetailsPanel({ model }: { model: JobDetailModel }) {
  return (
    <Panel title="Details">
      <DetailList>
        {model.rows.map((row) => (
          <DetailRow key={row.label} label={row.label}>
            <RowValue row={row} />
          </DetailRow>
        ))}
      </DetailList>
    </Panel>
  );
}

const styles = StyleSheet.create({
  place: { flexDirection: "row", alignItems: "flex-start", gap: 12 },
  placeText: { flexShrink: 1, gap: 2 },
  tile: {
    width: 32,
    height: 32,
    alignItems: "center",
    justifyContent: "center",
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.lg,
    backgroundColor: colors.surface,
  },
  muted: { color: colors.muted },
  ink: { color: colors.ink },
  timeline: {
    marginTop: 16,
    marginLeft: 0,
    paddingLeft: 14,
    gap: 10,
    borderLeftWidth: 1,
    borderLeftColor: colors.border,
  },
  event: { flexDirection: "row", alignItems: "flex-start", gap: 16 },
  time: { color: colors.muted, flexShrink: 0 },
  eventText: { color: colors.ink, flexShrink: 1 },
  value: { color: colors.ink, textAlign: "right" },
  code: {
    paddingHorizontal: 6,
    borderRadius: radii.sm,
    backgroundColor: colors.soft,
    color: colors.ink,
  },
});
