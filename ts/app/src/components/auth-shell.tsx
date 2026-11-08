// The signed-out frame: the brand above one centred card on the page grey,
// and a full-width white column on a phone. The card starts with the notice
// of a failed session check, when a check failed.

import type { ReactNode } from "react";
import { ScrollView, StyleSheet, Text, View } from "react-native";

import { colors, radii } from "@ciproxy/tokens";

import { Brand } from "./brand.tsx";
import { SessionRefreshNotice } from "./session-notice.tsx";
import { text } from "./text.ts";
import { useViewport } from "./viewport.ts";

export function AuthShell({
  children,
  footer,
}: {
  children: ReactNode;
  footer?: ReactNode;
}) {
  const mobile = useViewport() === "mobile";
  return (
    <ScrollView
      style={[styles.page, mobile && styles.pageMobile]}
      contentContainerStyle={[styles.content, mobile && styles.contentMobile]}
    >
      <View style={mobile ? null : styles.brand}>
        <Brand />
      </View>
      <View style={[styles.card, mobile && styles.cardMobile]}>
        <SessionRefreshNotice />
        {children}
      </View>
      {footer ? (
        <Text
          style={[text.caption, styles.footer, mobile && styles.footerMobile]}
        >
          {footer}
        </Text>
      ) : null}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  page: { flex: 1, backgroundColor: colors.bg },
  pageMobile: { backgroundColor: colors.surface },
  content: {
    flexGrow: 1,
    alignItems: "center",
    gap: 24,
    paddingTop: 80,
    paddingHorizontal: 24,
    paddingBottom: 40,
  },
  contentMobile: {
    alignItems: "stretch",
    gap: 28,
    paddingTop: 28,
    paddingHorizontal: 20,
    paddingBottom: 24,
  },
  brand: { alignItems: "center" },
  card: {
    width: "100%",
    maxWidth: 400,
    gap: 20,
    padding: 28,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.xl,
    backgroundColor: colors.surface,
  },
  cardMobile: { maxWidth: undefined, padding: 0, borderWidth: 0 },
  footer: { maxWidth: 400, textAlign: "center" },
  footerMobile: { maxWidth: undefined },
});
