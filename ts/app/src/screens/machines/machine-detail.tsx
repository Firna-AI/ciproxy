// One own machine: its slots and warm runners, health, cache, and the monthly
// cost that the savings report uses. Every role sees the machine; only an
// Admin drains, removes, clears, or changes it. The page reads the machines
// again while it shows them, and a failed read keeps the last machine under
// the shared refresh warning. After Remove, it returns to the list, which no
// longer holds the machine.

import { useEffect, useState } from "react";
import { Link } from "expo-router";
import { StyleSheet, View } from "react-native";

import { ActionLink } from "../../components/actions.tsx";
import {
  FeatureHeader,
  StatusLabel,
  TextLink,
} from "../../components/feature-parts.tsx";
import { MachinesIcon } from "../../components/icons.tsx";
import {
  ReadFailure,
  StateBlock,
  WaitingLine,
} from "../../components/page-state.tsx";
import { useViewport } from "../../components/viewport.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { overflowSummary } from "../../state/machines-overflow.ts";
import { machinesServices } from "../../state/machines-services.ts";
import { machinesPollInterval } from "../../state/machines.ts";
import {
  costEdit,
  isCommand,
  warmControl,
} from "../../view/machines-controls.ts";
import { detailCopy, listCopy, statusWords } from "../../view/machines-copy.ts";
import { machineSubtitle } from "../../view/machines-detail.ts";
import { canClearCache } from "../../view/machines-panels.ts";
import {
  jobHistoryHref,
  machinesHref,
  slotJobHref,
} from "../../view/machines-routes.ts";
import { slotsModel } from "../../view/machines-slots.ts";
import { machineStatus } from "../../view/machines.ts";
import type { WorkspaceContext } from "../workspace-page.tsx";
import { CachePanel, ClearCacheDialog } from "./machine-cache.tsx";
import { CostPanel } from "./machine-cost.tsx";
import { HealthPanel } from "./machine-health.tsx";
import { HeaderActions, MachineNotices } from "./machine-notices.tsx";
import { SlotsPanel } from "./machine-slots.tsx";
import { statusTones } from "./machine-status.ts";
import { MachinesRefreshNotice } from "./machines-refresh.tsx";

export function MachineDetail({
  context,
  machineId,
}: {
  context: WorkspaceContext;
  machineId: string;
}) {
  const services = useServices();
  const workspaceId = context.workspace.id;
  const stores = machinesServices(services);
  const store = stores.machines(workspaceId);
  const overflowStore = stores.overflow(workspaceId);
  const changes = stores.changes(workspaceId, machineId);
  const snapshot = useValue(store.state);
  const overflow = useValue(overflowStore.state);
  const change = useValue(changes.state);
  const viewport = useViewport();
  const [clearing, setClearing] = useState(false);
  const admin = context.role === "admin";
  const list = machinesHref(workspaceId);
  const removed = change.status === "done" && isCommand(change, "remove");
  useEffect(() => {
    void store.load();
    void overflowStore.ensure();
    return store.watch(machinesPollInterval);
  }, [store, overflowStore]);
  useEffect(() => {
    changes.dismiss();
    return () => changes.dismiss();
  }, [changes]);
  useEffect(() => {
    if (removed) services.navigator.go(list, { replace: true });
  }, [removed, services, list]);

  const crumb = <TextLink href={list} label={detailCopy.crumb} muted />;
  if (snapshot.status === "loading" || removed) {
    return (
      <>
        {crumb}
        <WaitingLine>{listCopy.loading}</WaitingLine>
      </>
    );
  }
  if (snapshot.status === "failed") {
    return (
      <>
        {crumb}
        <ReadFailure
          subject={listCopy.subject}
          onRetry={() => void store.load()}
        />
      </>
    );
  }
  const machine = snapshot.machines.find((item) => item.id === machineId);
  if (machine === undefined) {
    return (
      <>
        {crumb}
        <MachinesRefreshNotice store={store} />
        <StateBlock
          icon={MachinesIcon}
          title={detailCopy.notFound}
          action={
            <Link href={list} push asChild>
              <ActionLink
                label={listCopy.openMachines}
                href={list}
                tone="primary"
              />
            </Link>
          }
        />
      </>
    );
  }
  const status = machineStatus(machine);
  const now = services.env.now();
  const wide = viewport === "wide";
  const clearBusy =
    change.status === "busy" && isCommand(change, "clear-cache");
  const clearFailure =
    change.status === "failed" && isCommand(change, "clear-cache")
      ? change.failure
      : null;
  return (
    <>
      <FeatureHeader
        crumb={{ label: detailCopy.crumb, href: list }}
        title={machine.hostname}
        mono
        status={
          <StatusLabel tone={statusTones[status]}>
            {statusWords[status]}
          </StatusLabel>
        }
        subtitle={machineSubtitle(machine)}
        actions={
          admin ? (
            <HeaderActions
              machine={machine}
              change={change}
              onCommand={(command) =>
                void changes.run({ kind: "command", command })
              }
            />
          ) : undefined
        }
      />
      <MachinesRefreshNotice store={store} />
      <MachineNotices
        machine={machine}
        workspaceId={workspaceId}
        admin={admin}
        change={change}
        now={now}
      />
      <View style={[styles.grid, wide && styles.gridWide]}>
        <View style={[styles.column, wide && styles.columnWide]}>
          <SlotsPanel
            machine={machine}
            model={slotsModel(
              machine,
              snapshot.machines,
              overflowSummary(overflow),
            )}
            jobsHref={jobHistoryHref(workspaceId)}
            jobHref={(jobId) => slotJobHref(workspaceId, jobId)}
            warm={warmControl(machine, change, changes, admin)}
          />
        </View>
        <View style={[styles.column, wide && styles.columnWide]}>
          <HealthPanel machine={machine} now={now} />
          <CachePanel
            machine={machine}
            onClear={
              canClearCache(machine, admin)
                ? () => {
                    changes.dismiss();
                    setClearing(true);
                  }
                : null
            }
          />
          <CostPanel
            machine={machine}
            edit={admin ? costEdit(change, changes) : null}
          />
        </View>
      </View>
      <ClearCacheDialog
        machine={machine}
        visible={clearing}
        busy={clearBusy}
        failure={clearFailure}
        onCancel={() => {
          changes.dismiss();
          setClearing(false);
        }}
        onConfirm={() => {
          void changes
            .run({ kind: "command", command: "clear-cache" })
            .then((ok) => {
              if (ok) setClearing(false);
            });
        }}
      />
    </>
  );
}

const styles = StyleSheet.create({
  grid: { gap: 16 },
  gridWide: { flexDirection: "row", alignItems: "flex-start" },
  column: { gap: 16 },
  columnWide: { flex: 1, minWidth: 0 },
});
