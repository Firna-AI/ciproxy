// Where a job runs or ran: an own machine in the monospace face, with its
// slot while the job runs, or Boat. A finished job that no runner took says
// so, and a place that the API leaves out says "Unknown".

import { StyleSheet, Text, View } from "react-native";

import { colors } from "@ciproxy/tokens";

import { BoatIcon, MachinesIcon } from "../../components/icons.tsx";
import { text } from "../../components/text.ts";
import type { Place } from "../../view/jobs-format.ts";

export function PlaceLabel({ place, slot }: { place: Place; slot: boolean }) {
  switch (place.kind) {
    case "machine":
      return (
        <View style={styles.row}>
          <MachinesIcon size={14} color={colors.muted} />
          <Text style={[text.small, text.mono, styles.name]} numberOfLines={1}>
            {place.name}
          </Text>
          {slot && place.slot !== null ? (
            <Text style={[text.small, styles.muted]}>{place.slot}</Text>
          ) : null}
        </View>
      );
    case "boat":
      return (
        <View style={styles.row}>
          <BoatIcon size={14} color={colors.muted} />
          <Text style={text.small}>Boat</Text>
        </View>
      );
    case "none":
      return <Text style={[text.small, styles.muted]}>No runner</Text>;
    case "unknown":
      return <Text style={[text.small, styles.muted]}>Unknown</Text>;
  }
}

const styles = StyleSheet.create({
  row: { flexDirection: "row", alignItems: "center", gap: 8, minWidth: 0 },
  name: { color: colors.ink, flexShrink: 1 },
  muted: { color: colors.muted },
});
