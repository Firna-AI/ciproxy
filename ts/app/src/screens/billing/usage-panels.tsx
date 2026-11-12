// The panels of Plan and usage: the plan with its actions, the use of this
// period against the plan, and the skeleton while they load. A Viewer sees
// the same panels and reads who can pay in place of each payment action.

import type { ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";
import { Link } from "expo-router";
import { SkeletonBar, SkeletonGroup } from "@firna/ui/skeleton";

import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { Meter, MeterLine } from "../../components/billing/meter.tsx";
import { ExternalIcon } from "../../components/icons.tsx";
import { Panel } from "../../components/panel.tsx";
import { RoleNote } from "../../components/role-note.tsx";
import { text } from "../../components/text.ts";
import { useViewport } from "../../components/viewport.ts";
import type { ActionSnapshot } from "../../state/billing-actions.ts";
import { askPayer } from "../../view/billing-format.ts";
import { plansHref } from "../../view/billing-routes.ts";
import type { UsageModel } from "../../view/billing-usage.ts";

export function PlanPanel({
  model,
  workspaceId,
  canPay,
  portal,
  onPortal,
}: {
  model: UsageModel;
  workspaceId: string;
  canPay: boolean;
  portal: ActionSnapshot;
  onPortal: () => void;
}) {
  const { plan } = model;
  const plans = plansHref(workspaceId);
  const compare = (primary: boolean) => (
    <Link href={plans} asChild>
      <ActionLink
        label={plan.paid && canPay ? "Change plan" : "Compare plans"}
        href={plans}
        tone={primary ? "primary" : "secondary"}
      />
    </Link>
  );
  return (
    <Panel title="Plan">
      <View style={styles.stack}>
        <View style={styles.tight}>
          <Text style={text.title}>{plan.name}</Text>
          <Text style={text.lead}>{plan.price}</Text>
          <Text style={text.small}>{plan.capacity}</Text>
        </View>
        <View style={styles.actions}>
          {compare(canPay && !plan.paid)}
          {canPay && plan.paid ? (
            <ActionButton
              label="Invoices and card"
              icon={ExternalIcon}
              busy={portal.status === "working"}
              onPress={onPortal}
            />
          ) : null}
          {canPay ? null : <RoleNote>{askPayer}</RoleNote>}
        </View>
        {canPay && plan.paid ? (
          <Text style={text.caption}>
            {portal.status === "failed"
              ? "Unable to open the Stripe billing portal. Try again."
              : "Invoices and the card are in the Stripe billing portal."}
          </Text>
        ) : null}
      </View>
    </Panel>
  );
}

export function PeriodPanel({ model }: { model: UsageModel }) {
  return (
    <Panel title="This period">
      <View style={styles.stack}>
        <Text style={text.caption}>{model.period}</Text>
        <Meter model={model.starts} />
        <Meter model={model.machines} />
        {model.organizations ? (
          <MeterLine
            label="Organizations"
            value={model.organizations.value}
            detail={model.organizations.detail}
          />
        ) : null}
        <Text style={text.caption}>
          Counts the starts and machines of every organization of this
          workspace.
        </Text>
      </View>
    </Panel>
  );
}

/** Two panels side by side on a wide window, one above the other else. */
export function PanelPair({ children }: { children: ReactNode }) {
  const wide = useViewport() === "wide";
  return <View style={wide ? styles.pair : styles.column}>{children}</View>;
}

/** One panel of a pair, which shares the width with the other one. */
export function Half({ children }: { children: ReactNode }) {
  return <View style={styles.half}>{children}</View>;
}

export function LoadingUsage() {
  return (
    <PanelPair>
      {[0, 1].map((key) => (
        <Half key={key}>
          <Panel>
            <SkeletonGroup direction="column" gap={12}>
              <SkeletonBar width="40%" height={16} />
              <SkeletonBar width="70%" height={12} />
              <SkeletonBar width="100%" height={8} />
              <SkeletonBar width="100%" height={8} />
            </SkeletonGroup>
          </Panel>
        </Half>
      ))}
    </PanelPair>
  );
}

const styles = StyleSheet.create({
  stack: { gap: 16 },
  tight: { gap: 4 },
  actions: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    gap: 8,
  },
  pair: { flexDirection: "row", alignItems: "stretch", gap: 16 },
  column: { gap: 16 },
  half: { flex: 1, minWidth: 0 },
});
