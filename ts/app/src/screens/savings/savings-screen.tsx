// Savings: what the jobs of one UTC range cost with ciproxy, against the same
// jobs on Blacksmith and on GitHub-hosted runners. The range lives in the
// address and stays above the report, also when the range has no jobs. Every
// role reads the report, and only an Admin enters what a machine costs.

import { Link, useLocalSearchParams } from "expo-router";
import { StyleSheet, View } from "react-native";
import { SegmentedControl } from "@firna/ui/segmented";
import { SkeletonBar, SkeletonGroup } from "@firna/ui/skeleton";

import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { SavingsIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import {
  PageHeader,
  ReadFailure,
  StateBlock,
  WaitingLine,
} from "../../components/page-state.tsx";
import { Panel } from "../../components/panel.tsx";
import { RoleNote } from "../../components/role-note.tsx";
import { useViewport } from "../../components/viewport.ts";
import type { SavingsPage, SavingsRange } from "../../api/savings-types.ts";
import { workspaceHref } from "../../routing/pages.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { savingsNotices } from "../../view/savings-costs.ts";
import {
  emptyReport,
  rangeLabels,
  rangeOf,
  rangeOrder,
  savingsHref,
  savingsSubtitle,
  totalStarts,
} from "../../view/savings-report.ts";
import { setupComplete } from "../../view/setup-steps.ts";
import {
  WorkspacePageView,
  type WorkspaceContext,
} from "../workspace-page.tsx";
import { CostPanel, ReportPanel } from "./savings-panels.tsx";
import { useSavings } from "./savings-hooks.ts";
import { StartsPanel } from "./starts-chart.tsx";

const ranges = rangeOrder.map((value) => ({
  value,
  label: rangeLabels[value],
}));

/** Set up ciproxy, for an Admin while Setup is open. */
function SetupLink({ workspaceId }: { workspaceId: string }) {
  const services = useServices();
  const snapshot = useValue(services.setup(workspaceId).state);
  if (snapshot.status !== "ready") return null;
  if (setupComplete(snapshot.setup, snapshot.machines)) return null;
  const href = workspaceHref("setup", workspaceId);
  return (
    <Link href={href} asChild>
      <ActionLink label="Set up ciproxy" href={href} tone="primary" />
    </Link>
  );
}

function Report({
  page,
  context,
}: {
  page: SavingsPage;
  context: WorkspaceContext;
}) {
  const wide = useViewport() === "wide";
  const machines = workspaceHref("machines", context.workspace.id);
  if (totalStarts(page) === 0n) {
    const empty = emptyReport(page.range);
    return (
      <Panel>
        <StateBlock
          icon={SavingsIcon}
          title={empty.title}
          action={
            empty.setup && context.role === "admin" ? (
              <SetupLink workspaceId={context.workspace.id} />
            ) : null
          }
        >
          {empty.body}
        </StateBlock>
      </Panel>
    );
  }
  return (
    <>
      {savingsNotices(page).map((notice) => (
        <Notice
          key={notice.key}
          tone={notice.tone}
          title={notice.title}
          actions={
            !notice.addCosts ? null : page.can_add_machine_costs ? (
              <Link href={machines} asChild>
                <ActionLink label="Add costs" href={machines} size="sm" />
              </Link>
            ) : (
              <RoleNote>Ask an Admin</RoleNote>
            )
          }
        >
          {notice.body}
        </Notice>
      ))}
      <ReportPanel page={page} />
      <View style={[styles.grid, wide && styles.gridWide]}>
        <View style={wide ? styles.cellWide : undefined}>
          <CostPanel page={page} />
        </View>
        <View style={wide ? styles.cellWide : undefined}>
          <StartsPanel page={page} />
        </View>
      </View>
    </>
  );
}

function SavingsView({ context }: { context: WorkspaceContext }) {
  const { navigator } = useServices();
  const params = useLocalSearchParams<{ range?: string | string[] }>();
  const range = rangeOf(params.range);
  const { snapshot, retry } = useSavings(context.workspace.id, range);
  const choose = (next: SavingsRange) =>
    navigator.go(savingsHref(context.workspace.id, next), { replace: true });
  return (
    <>
      <PageHeader
        title="Savings"
        subtitle={
          snapshot.status === "ready"
            ? savingsSubtitle(snapshot.page)
            : undefined
        }
      />
      {snapshot.status === "failed" ? null : (
        <View style={styles.ranges}>
          <SegmentedControl<SavingsRange>
            options={ranges}
            value={range}
            onChange={choose}
            accessibilityLabel="Period"
          />
        </View>
      )}
      {snapshot.status === "loading" ? (
        <Panel>
          <SkeletonGroup direction="column" gap={14}>
            <SkeletonBar width={140} height={44} />
            <SkeletonBar width="70%" height={14} />
            <SkeletonBar width="90%" height={20} />
            <SkeletonBar width="60%" height={20} />
            <SkeletonBar width="40%" height={20} />
          </SkeletonGroup>
        </Panel>
      ) : snapshot.status === "failed" ? (
        <Panel>
          <ReadFailure subject="the savings report" onRetry={retry} />
        </Panel>
      ) : (
        <>
          {snapshot.refreshFailure ? (
            <Notice
              tone="warning"
              title="Unable to refresh the report"
              actions={
                <ActionButton label="Try again" size="sm" onPress={retry} />
              }
            >
              The page shows the last report that it read.
            </Notice>
          ) : null}
          {snapshot.page.refresh_complete ? null : (
            <WaitingLine>Updating the report…</WaitingLine>
          )}
          <Report page={snapshot.page} context={context} />
        </>
      )}
    </>
  );
}

export function SavingsScreen() {
  return (
    <WorkspacePageView page="savings">
      {(context) => <SavingsView context={context} />}
    </WorkspacePageView>
  );
}

const styles = StyleSheet.create({
  ranges: { alignSelf: "flex-start", maxWidth: "100%" },
  grid: { gap: 16 },
  gridWide: { flexDirection: "row", alignItems: "flex-start" },
  /**
   * Equal columns only in a row. In a column, a zero basis would let a panel
   * collapse to no height, so the next panel would cover it.
   */
  cellWide: { flexGrow: 1, flexShrink: 1, flexBasis: 0, minWidth: 0 },
});
