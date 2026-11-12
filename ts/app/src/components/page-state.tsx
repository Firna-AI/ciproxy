// The states of a page around its content: the page header, loading, a
// failed read with Try again, and the centred block of an empty, finished,
// warning, or unavailable state.

import type { ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";
import type { IconComponent } from "@firna/ui";
import { Spinner } from "@firna/ui/spinner";

import { colors, radii, statusColors } from "@ciproxy/tokens";

import { useServices } from "../services/context.tsx";
import type { SessionSnapshot } from "../state/session.ts";
import { ActionButton } from "./actions.tsx";
import { AlertIcon } from "./icons.tsx";
import { text } from "./text.ts";

export function PageHeader({
  title,
  subtitle,
}: {
  title: string;
  subtitle?: string;
}) {
  return (
    <View style={styles.header}>
      <Text role="heading" aria-level={1} style={text.pageTitle}>
        {title}
      </Text>
      {subtitle ? <Text style={text.lead}>{subtitle}</Text> : null}
    </View>
  );
}

export function Busy({ size = 16 }: { size?: number }) {
  return (
    <Spinner size={size} color={colors.muted} accessibilityLabel="Loading" />
  );
}

export function WaitingLine({ children }: { children: string }) {
  return (
    <View style={styles.waiting} role="status">
      <Busy />
      <Text style={text.small}>{children}</Text>
    </View>
  );
}

/** The look of a finished or a warning state block. */
export type StateTone = "success" | "warning";

const iconColors = {
  neutral: colors.ink2,
  success: statusColors.ok,
  warning: colors.amberDeep,
  danger: colors.roseDeep,
} as const;

export function StateBlock({
  icon: Icon,
  danger = false,
  tone,
  title,
  children,
  action,
}: {
  icon: IconComponent;
  danger?: boolean;
  /** A finished or a warning state. A danger state keeps its own look. */
  tone?: StateTone;
  title: string;
  children?: ReactNode;
  action?: ReactNode;
}) {
  const look = danger ? "danger" : (tone ?? "neutral");
  return (
    <View style={styles.block} role={danger ? "alert" : undefined}>
      <View
        style={[
          styles.blockIcon,
          look === "danger" && styles.blockIconDanger,
          look === "warning" && styles.blockIconWarning,
        ]}
      >
        <Icon size={18} color={iconColors[look]} />
      </View>
      <Text role="heading" aria-level={2} style={text.heading}>
        {title}
      </Text>
      {children ? (
        <Text style={[text.small, styles.blockBody]}>{children}</Text>
      ) : null}
      {action ? <View style={styles.blockAction}>{action}</View> : null}
    </View>
  );
}

/** A read that failed, with Try again. */
export function ReadFailure({
  subject,
  onRetry,
}: {
  subject: string;
  onRetry: () => void;
}) {
  return (
    <StateBlock
      icon={AlertIcon}
      danger
      title={`Unable to load ${subject}`}
      action={
        <ActionButton label="Try again" tone="primary" onPress={onRetry} />
      }
    >
      Check the connection, then try again.
    </StateBlock>
  );
}

/** A whole page that waits for the session, or failed to read it. */
export function SessionWaiting({ snapshot }: { snapshot: SessionSnapshot }) {
  const { session } = useServices();
  return (
    <View style={styles.page}>
      {snapshot.status === "failed" ? (
        <ReadFailure subject="ciproxy" onRetry={() => void session.refresh()} />
      ) : (
        <Busy size={24} />
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  header: { gap: 4 },
  waiting: { flexDirection: "row", alignItems: "center", gap: 8 },
  page: {
    flex: 1,
    alignItems: "center",
    justifyContent: "center",
    padding: 24,
    backgroundColor: colors.bg,
  },
  block: {
    alignItems: "center",
    gap: 6,
    paddingVertical: 40,
    paddingHorizontal: 16,
  },
  blockIcon: {
    width: 40,
    height: 40,
    alignItems: "center",
    justifyContent: "center",
    marginBottom: 6,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.lg,
    backgroundColor: colors.surface,
  },
  blockIconDanger: {
    borderColor: colors.roseSoft,
    backgroundColor: colors.roseSoft,
  },
  blockIconWarning: {
    borderColor: colors.amberSoft,
    backgroundColor: colors.amberSoft,
  },
  blockBody: { maxWidth: 400, textAlign: "center" },
  blockAction: {
    flexDirection: "row",
    flexWrap: "wrap",
    justifyContent: "center",
    gap: 8,
    marginTop: 8,
  },
});
