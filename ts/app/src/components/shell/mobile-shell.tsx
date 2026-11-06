// The phone frame: a top bar with the workspace switch and the account, a
// strip that leads back to Setup while it is open, the page, and a tab bar.

import { Link } from "expo-router";
import { ScrollView, StyleSheet, Text, View } from "react-native";

import { colors, fonts, navColors } from "@ciproxy/tokens";

import { BrandMark } from "../brand.tsx";
import { ChevronIcon, SetupIcon, SwitchIcon } from "../icons.tsx";
import { text } from "../text.ts";
import type { ShellFrameProps } from "./desktop-shell.tsx";
import { AccountMenu, WorkspaceSwitch } from "./menus.tsx";
import { NavLink } from "./nav-link.tsx";

function SetupStrip({ href, count }: { href: string; count: string }) {
  return (
    <Link href={href} style={styles.strip}>
      <SetupIcon size={15} color={colors.ink} />
      <Text style={[text.body, text.strong, styles.stripLabel]}>
        Finish setup
      </Text>
      <Text style={styles.count}>{count}</Text>
      <ChevronIcon size={15} color={navColors.icon} />
    </Link>
  );
}

export function MobileShell(props: ShellFrameProps) {
  const { session, workspace, role, page, items, actions, children } = props;
  const setup = items.find((item) => item.key === "setup");
  const tabs = items.filter((item) => item.key !== "setup");
  return (
    <View style={styles.shell}>
      <View style={styles.top}>
        <WorkspaceSwitch
          session={session}
          workspace={workspace}
          onAddOrganization={actions.addOrganization}
          trigger={
            <View style={styles.switch}>
              <BrandMark size={18} />
              <View style={styles.divider} />
              <Text style={[text.body, text.strong]} numberOfLines={1}>
                {workspace.name}
              </Text>
              <SwitchIcon size={14} color={navColors.icon} />
            </View>
          }
        />
        <AccountMenu
          session={session}
          workspace={workspace}
          role={role}
          onClaim={actions.claimAdmin}
          onSignOut={actions.signOut}
          size={26}
        />
      </View>
      {setup && page !== "setup" && setup.count ? (
        <SetupStrip href={setup.href} count={setup.count} />
      ) : null}
      <ScrollView style={styles.scroll} contentContainerStyle={styles.main}>
        {children}
      </ScrollView>
      <View style={styles.tabs} role="navigation" aria-label="Main">
        {tabs.map((item) => (
          <View key={item.key} style={styles.tab}>
            <NavLink item={item} active={item.key === page} variant="tab" />
          </View>
        ))}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  shell: { flex: 1, backgroundColor: colors.bg },
  top: {
    height: 52,
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    paddingLeft: 8,
    paddingRight: 12,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
    backgroundColor: colors.surface,
  },
  switch: { flexDirection: "row", alignItems: "center", gap: 10, minWidth: 0 },
  divider: {
    width: 1,
    height: 18,
    backgroundColor: colors.ink,
    opacity: 0.22,
    transform: [{ rotate: "20deg" }],
  },
  strip: {
    display: "flex",
    flexDirection: "row",
    alignItems: "center",
    gap: 8,
    paddingVertical: 10,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
    backgroundColor: colors.surface,
  },
  stripLabel: { flex: 1 },
  count: { color: navColors.label, fontFamily: fonts.mono, fontSize: 11 },
  scroll: { flex: 1 },
  main: { gap: 16, paddingTop: 16, paddingHorizontal: 16, paddingBottom: 24 },
  tabs: {
    flexDirection: "row",
    paddingTop: 6,
    paddingHorizontal: 4,
    paddingBottom: 10,
    borderTopWidth: 1,
    borderTopColor: colors.border,
    backgroundColor: colors.surface,
  },
  tab: { flex: 1, alignItems: "center" },
});
