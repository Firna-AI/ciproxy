// The line under a machine that connected and gets its first job image: a
// small spinner beside the words, which wrap under one another. Setup and
// Add a machine show it under what the agent reported. No text promises a
// time, because a slow link makes the download long.

import { StyleSheet, Text, View } from "react-native";

import { colors } from "@ciproxy/tokens";

import { Busy } from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import { imageCopy } from "../../view/machines-copy.ts";

export function ImageLine() {
  return (
    <View style={styles.line}>
      <View style={styles.spinner}>
        <Busy size={14} />
      </View>
      <Text style={[text.caption, styles.words]}>{imageCopy.line}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  line: {
    flexDirection: "row",
    alignItems: "flex-start",
    gap: 8,
    marginTop: 4,
  },
  spinner: { flexShrink: 0, marginTop: 1 },
  words: { flex: 1, minWidth: 0, color: colors.ink2 },
});
