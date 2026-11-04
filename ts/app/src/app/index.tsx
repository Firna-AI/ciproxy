// The dashboard home. Until the dashboard has its screens, it says what
// ciproxy does, in the frame of the signed-out screens of the mockups: the
// brand above one centred card on the page grey.

import { Body, H1 } from "@firna/ui/typography";
import { ScrollView, StyleSheet, View } from "react-native";

import { colors, radii } from "@ciproxy/tokens";

import { Brand } from "../components/brand.tsx";

export default function Dashboard() {
  return (
    <ScrollView style={styles.page} contentContainerStyle={styles.content}>
      <Brand />
      <View style={styles.card}>
        <H1 style={styles.title}>
          Run GitHub Actions{"\n"}on your own machines
        </H1>
        <Body color="secondary" style={styles.lead}>
          ciproxy starts each job on your machines first, and on Boat when they
          are busy.
        </Body>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  page: { flex: 1, backgroundColor: colors.bg },
  content: {
    flexGrow: 1,
    alignItems: "center",
    gap: 24,
    paddingTop: 80,
    paddingHorizontal: 24,
    paddingBottom: 40,
  },
  card: {
    width: "100%",
    maxWidth: 400,
    gap: 6,
    padding: 28,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.xl,
    backgroundColor: colors.surface,
  },
  title: {
    fontSize: 22,
    fontWeight: "600",
    letterSpacing: -0.44,
    lineHeight: 28,
  },
  lead: { fontSize: 14, lineHeight: 20 },
});
