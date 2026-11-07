// The desktop frame: a white sidebar with the main navigation beside a top
// bar with the workspace switch and the page, and the page below. In a
// narrow window the sidebar becomes a rail of icons.

import type { ReactNode } from "react";
import { ScrollView, StyleSheet, Text, View } from "react-native";
import { Avatar } from "@firna/ui/avatar";

import { colors, navColors } from "@ciproxy/tokens";

import type {
  SessionView,
  WorkspaceRole,
  WorkspaceView,
} from "../../api/schema.ts";
import type { WorkspacePage } from "../../routing/pages.ts";
import { initial } from "../../view/format.ts";
import { pageTitles, type NavItem } from "../../view/shell.ts";
import { Brand, BrandMark } from "../brand.tsx";
import { SwitchIcon } from "../icons.tsx";
import { text } from "../text.ts";
import { AccountMenu, WorkspaceSwitch } from "./menus.tsx";
import { DocsLink, NavLink } from "./nav-link.tsx";
import type { useShellActions } from "./shell-actions.ts";

export type ShellFrameProps = {
  session: SessionView;
  workspace: WorkspaceView;
  role: WorkspaceRole;
  page: WorkspacePage;
  items: readonly NavItem[];
  actions: ReturnType<typeof useShellActions>;
  children: ReactNode;
};

export function DesktopShell({
  rail,
  ...props
}: ShellFrameProps & { rail: boolean }) {
  const { session, workspace, role, page, items, actions, children } = props;
  return (
    <View style={styles.shell}>
      <View style={[styles.side, rail && styles.rail]}>
        <View style={[styles.brand, rail && styles.brandRail]}>
          {rail ? <BrandMark /> : <Brand />}
        </View>
        <View
          style={[styles.nav, rail && styles.navRail]}
          role="navigation"
          aria-label="Main"
        >
          {items.map((item) => (
            <NavLink
              key={item.key}
              item={item}
              active={item.key === page}
              variant={rail ? "rail" : "side"}
            />
          ))}
        </View>
        <View style={[styles.foot, rail && styles.navRail]}>
          <DocsLink rail={rail} />
        </View>
      </View>
      <View style={styles.body}>
        <View style={[styles.topbar, rail && styles.topbarRail]}>
          <View style={styles.crumbs}>
            <WorkspaceSwitch
              session={session}
              workspace={workspace}
              onAddOrganization={actions.addOrganization}
              trigger={
                <View style={styles.switch}>
                  <Avatar
                    label={initial(workspace.name)}
                    size={22}
                    shape="square"
                    decorative
                  />
                  <Text
                    style={[text.body, text.strong, styles.name]}
                    numberOfLines={1}
                  >
                    {workspace.name}
                  </Text>
                  <SwitchIcon size={14} color={navColors.icon} />
                </View>
              }
            />
            <Text style={styles.slash} aria-hidden>
              /
            </Text>
            <Text style={[text.body, text.strong]} numberOfLines={1}>
              {pageTitles[page]}
            </Text>
          </View>
          <AccountMenu
            session={session}
            workspace={workspace}
            role={role}
            onClaim={actions.claimAdmin}
            onSignOut={actions.signOut}
            size={28}
          />
        </View>
        <ScrollView
          style={styles.scroll}
          contentContainerStyle={[styles.main, rail && styles.mainRail]}
        >
          {children}
        </ScrollView>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  shell: { flex: 1, flexDirection: "row", backgroundColor: colors.bg },
  side: {
    width: 240,
    gap: 12,
    paddingHorizontal: 12,
    paddingBottom: 12,
    borderRightWidth: 1,
    borderRightColor: colors.border,
    backgroundColor: colors.surface,
  },
  rail: { width: 60, paddingHorizontal: 8, alignItems: "center" },
  brand: {
    height: 56,
    justifyContent: "center",
    marginHorizontal: -12,
    paddingHorizontal: 20,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
  },
  brandRail: {
    width: 60,
    marginHorizontal: -8,
    paddingHorizontal: 0,
    alignItems: "center",
  },
  nav: { flex: 1, gap: 1 },
  navRail: { alignItems: "center" },
  foot: {
    paddingTop: 10,
    borderTopWidth: 1,
    borderTopColor: colors.border,
    alignSelf: "stretch",
  },
  body: { flex: 1, minWidth: 0 },
  topbar: {
    height: 56,
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
    paddingLeft: 16,
    paddingRight: 24,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
    backgroundColor: colors.surface,
  },
  topbarRail: { paddingLeft: 8, paddingRight: 16 },
  crumbs: {
    flexDirection: "row",
    alignItems: "center",
    gap: 6,
    minWidth: 0,
    flexShrink: 1,
  },
  switch: {
    flexDirection: "row",
    alignItems: "center",
    gap: 8,
    flexShrink: 1,
    minWidth: 0,
  },
  name: { flexShrink: 1 },
  slash: { color: colors.border2, fontSize: 20, fontWeight: "300" },
  scroll: { flex: 1 },
  main: { gap: 20, paddingTop: 28, paddingHorizontal: 40, paddingBottom: 48 },
  mainRail: { paddingTop: 22, paddingHorizontal: 24, paddingBottom: 32 },
});
