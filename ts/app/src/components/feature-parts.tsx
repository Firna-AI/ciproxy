// Generic parts of the feature pages, after the mockups: a panel with a
// hairline border, a detail row, a status dot beside its word, a text link,
// the page header with its breadcrumb and actions, and a dialog that becomes
// a bottom sheet on a phone. Machines and Settings share them. A removal uses
// ActionButton with the danger tone. Colours, fonts, and radii come from the
// token package.

import {
  useState,
  type MouseEvent,
  type ReactNode,
  type RefObject,
} from "react";
import { Link } from "expo-router";
import { StyleSheet, Text, View } from "react-native";
import { useFocusRing } from "@firna/ui/focusRing";
import { WebModalFrame, type Focusable } from "@firna/ui/modal";
import { StatusDot } from "@firna/ui/status-dot";

import {
  colors,
  fonts,
  radii,
  statusColors,
  type StatusTone,
} from "@ciproxy/tokens";

import { text } from "./text.ts";
import { useViewport } from "./viewport.ts";

export function Panel({
  title,
  action,
  flush = false,
  children,
}: {
  title?: string;
  action?: ReactNode;
  /** The content reaches the border, as a table does. */
  flush?: boolean;
  children: ReactNode;
}) {
  const head =
    title === undefined && action === undefined ? null : (
      <View style={[styles.panelHead, flush && styles.panelHeadFlush]}>
        {title === undefined ? (
          <View />
        ) : (
          <Text role="heading" aria-level={2} style={styles.panelTitle}>
            {title}
          </Text>
        )}
        {action}
      </View>
    );
  return (
    <View style={[styles.panel, flush && styles.panelFlush]}>
      {head}
      {children}
    </View>
  );
}

export function DetailRow({
  label,
  first = false,
  children,
}: {
  label: string;
  first?: boolean;
  children: ReactNode;
}) {
  return (
    <View style={[styles.row, first && styles.rowFirst]}>
      <Text style={[text.small, styles.rowLabel]}>{label}</Text>
      <View style={styles.rowValue}>{children}</View>
    </View>
  );
}

/** A status: a small dot beside its word, never colour alone. */
export function StatusLabel({
  tone,
  muted = false,
  children,
}: {
  tone: StatusTone;
  muted?: boolean;
  children: string;
}) {
  return (
    <View style={styles.status}>
      <StatusDot color={statusColors[tone]} size="sm" />
      <Text style={[text.small, muted ? styles.muted : styles.ink]}>
        {children}
      </Text>
    </View>
  );
}

type AnchorProps = {
  label: string;
  mono: boolean;
  muted: boolean;
  /** Set by the router link. */
  href?: string;
  onClick?: (event: MouseEvent<HTMLAnchorElement>) => void;
};

function Anchor({ label, mono, muted, href, onClick }: AnchorProps) {
  const focus = useFocusRing();
  const [hovered, setHovered] = useState(false);
  return (
    <a
      href={href}
      onClick={onClick}
      {...focus.focusRingDomProps}
      onFocus={focus.onFocus}
      onBlur={focus.onBlur}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        color: muted && !hovered ? colors.muted : colors.ink,
        fontFamily: mono ? fonts.mono : fonts.sans,
        fontSize: 13,
        fontWeight: 500,
        lineHeight: "18px",
        textDecoration: hovered && !muted ? "underline" : "none",
        textUnderlineOffset: 3,
        overflowWrap: "anywhere",
        borderRadius: radii.sm,
        ...focus.focusRingVariables,
      }}
    >
      {label}
    </a>
  );
}

/** A text link. It pushes, so a change of the query alone adds history. */
export function TextLink({
  href,
  label,
  mono = false,
  muted = false,
}: {
  href: string;
  label: string;
  mono?: boolean;
  muted?: boolean;
}) {
  return (
    <Link href={href} push asChild>
      <Anchor label={label} mono={mono} muted={muted} />
    </Link>
  );
}

export function FeatureHeader({
  crumb,
  title,
  mono = false,
  status,
  subtitle,
  actions,
}: {
  crumb?: { label: string; href: string };
  title: string;
  mono?: boolean;
  status?: ReactNode;
  subtitle?: string;
  actions?: ReactNode;
}) {
  return (
    <View style={styles.header}>
      <View style={styles.headerText}>
        {crumb ? (
          <TextLink href={crumb.href} label={crumb.label} muted />
        ) : null}
        <View style={styles.titleRow}>
          <Text
            role="heading"
            aria-level={1}
            style={[text.pageTitle, mono && text.mono, styles.title]}
          >
            {title}
          </Text>
          {status}
        </View>
        {subtitle ? <Text style={text.caption}>{subtitle}</Text> : null}
      </View>
      {actions ? <View style={styles.actions}>{actions}</View> : null}
    </View>
  );
}

/**
 * A dialog over the page, and a bottom sheet on a phone. While its change
 * runs, it cannot close. Point `initialFocusRef` at the safe action of a
 * destructive confirmation.
 */
export function FeatureDialog({
  title,
  visible,
  busy = false,
  onClose,
  footer,
  initialFocusRef,
  children,
}: {
  title: string;
  visible: boolean;
  busy?: boolean;
  onClose: () => void;
  footer: ReactNode;
  initialFocusRef?: RefObject<Focusable | null>;
  children: ReactNode;
}) {
  const viewport = useViewport();
  return (
    <WebModalFrame
      title={title}
      visible={visible}
      onClose={onClose}
      closeDisabled={busy}
      dismissible={!busy}
      placement={viewport === "mobile" ? "bottom-sheet" : "center"}
      size="sm"
      footer={footer}
      initialFocusRef={initialFocusRef}
    >
      {children}
    </WebModalFrame>
  );
}

const styles = StyleSheet.create({
  panel: {
    gap: 12,
    padding: 16,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.xl,
    backgroundColor: colors.surface,
  },
  panelFlush: { padding: 0, gap: 0, overflow: "hidden" },
  panelHead: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
  },
  panelHeadFlush: {
    minHeight: 48,
    alignContent: "center",
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
  },
  panelTitle: {
    fontFamily: fonts.sans,
    fontSize: 14,
    fontWeight: "600",
    lineHeight: 20,
    color: colors.ink,
  },
  row: {
    flexDirection: "row",
    flexWrap: "wrap",
    justifyContent: "space-between",
    gap: 16,
    paddingVertical: 10,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
  rowFirst: { borderTopWidth: 0, paddingTop: 0 },
  rowLabel: { color: colors.muted },
  rowValue: { flexShrink: 1, alignItems: "flex-end" },
  status: { flexDirection: "row", alignItems: "center", gap: 6 },
  muted: { color: colors.muted },
  ink: { color: colors.ink },
  header: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
  },
  headerText: { flexGrow: 1, flexShrink: 1, flexBasis: 260, gap: 2 },
  titleRow: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    columnGap: 12,
    rowGap: 4,
  },
  title: { flexShrink: 1 },
  actions: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
});
