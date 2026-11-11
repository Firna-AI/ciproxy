// The own machines of the workspace: the figures, the table of machines with
// their slots, health, and cache, and the Overflow panel. Every role sees
// them; only an Admin adds a machine. The page reads the machines again while
// it shows them, and a failed refresh keeps the last read under the shared
// refresh warning with Try again.

import { useEffect } from "react";
import { Link } from "expo-router";
import { StyleSheet, View } from "react-native";
import { SkeletonBar } from "@firna/ui/skeleton";

import type { MachineView } from "../../api/schema.ts";
import { ActionLink } from "../../components/actions.tsx";
import { FeatureHeader, Panel } from "../../components/feature-parts.tsx";
import { MachinesIcon, PlusIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import {
  ReadFailure,
  StateBlock,
  WaitingLine,
} from "../../components/page-state.tsx";
import { useViewport } from "../../components/viewport.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { machinesServices } from "../../state/machines-services.ts";
import {
  machinesPollInterval,
  type MachinesSnapshot,
} from "../../state/machines.ts";
import {
  overflowSummary,
  type OverflowSnapshot,
} from "../../state/machines-overflow.ts";
import {
  elsewhereCopy,
  listCopy,
  noticeCopy,
} from "../../view/machines-copy.ts";
import { addMachineHref, machineHref } from "../../view/machines-routes.ts";
import {
  contactSince,
  elsewhereText,
  machineFigures,
} from "../../view/machines.ts";
import type { WorkspaceContext } from "../workspace-page.tsx";
import { FigureRow } from "./machine-figures.tsx";
import { MachineTable } from "./machine-table.tsx";
import { MachinesRefreshNotice } from "./machines-refresh.tsx";
import { OverflowPanel } from "./overflow-panel.tsx";

function AddMachineLink({ workspaceId }: { workspaceId: string }) {
  const href = addMachineHref(workspaceId);
  return (
    <Link href={href} push asChild>
      <ActionLink
        label={listCopy.addMachine}
        href={href}
        tone="primary"
        icon={PlusIcon}
      />
    </Link>
  );
}

function LoadingRows() {
  return (
    <View style={styles.loading}>
      <WaitingLine>{listCopy.loading}</WaitingLine>
      {[0, 1, 2].map((row) => (
        <SkeletonBar key={row} height={12} width={row === 2 ? "45%" : "70%"} />
      ))}
    </View>
  );
}

function OfflineNotices({
  machines,
  workspaceId,
  elsewhere,
  now,
}: {
  machines: readonly MachineView[];
  workspaceId: string;
  elsewhere: (machine: MachineView) => string;
  now: number;
}) {
  return machines
    .filter((machine) => machine.offline)
    .map((machine) => {
      const href = machineHref(workspaceId, machine.id);
      return (
        <Notice
          key={machine.id}
          tone="danger"
          title={noticeCopy.offlineTitle(machine.hostname)}
          actions={
            <Link href={href} push asChild>
              <ActionLink label={listCopy.openMachine} href={href} size="sm" />
            </Link>
          }
        >
          {`${noticeCopy.lostContact(contactSince(machine, now))} ${elsewhere(machine)}`}
        </Notice>
      );
    });
}

export function MachinesList({ context }: { context: WorkspaceContext }) {
  const services = useServices();
  const workspaceId = context.workspace.id;
  const stores = machinesServices(services);
  const store = stores.machines(workspaceId);
  const overflowStore = stores.overflow(workspaceId);
  const snapshot = useValue(store.state);
  const overflow = useValue(overflowStore.state);
  const viewport = useViewport();
  const admin = context.role === "admin";
  useEffect(() => {
    void store.load();
    void overflowStore.load();
    return store.watch(machinesPollInterval);
  }, [store, overflowStore]);

  const empty = snapshot.status === "ready" && snapshot.machines.length === 0;
  const showAdd = admin && !empty;
  return (
    <>
      <FeatureHeader
        title={listCopy.title}
        actions={
          showAdd ? <AddMachineLink workspaceId={workspaceId} /> : undefined
        }
      />
      <MachinesRefreshNotice store={store} />
      <Body
        snapshot={snapshot}
        context={context}
        onRetry={() => void store.load()}
        overflow={overflow}
        onOverflowRetry={() => void overflowStore.load()}
        compact={viewport === "mobile"}
      />
    </>
  );
}

function Body({
  snapshot,
  context,
  onRetry,
  overflow,
  onOverflowRetry,
  compact,
}: {
  snapshot: MachinesSnapshot;
  context: WorkspaceContext;
  onRetry: () => void;
  overflow: OverflowSnapshot;
  onOverflowRetry: () => void;
  compact: boolean;
}) {
  const services = useServices();
  const viewport = useViewport();
  const workspaceId = context.workspace.id;
  const admin = context.role === "admin";
  if (snapshot.status === "loading") {
    return (
      <Panel title={listCopy.ownMachines} flush>
        <LoadingRows />
      </Panel>
    );
  }
  if (snapshot.status === "failed") {
    return (
      <Panel>
        <ReadFailure subject={listCopy.subject} onRetry={onRetry} />
      </Panel>
    );
  }
  if (snapshot.machines.length === 0) {
    return (
      <Panel>
        <StateBlock
          icon={MachinesIcon}
          title={listCopy.emptyTitle}
          action={admin ? <AddMachineLink workspaceId={workspaceId} /> : null}
        >
          {listCopy.emptyBody}
        </StateBlock>
      </Panel>
    );
  }
  const { machines } = snapshot;
  const summary = overflowSummary(overflow);
  const figures = machineFigures(machines);
  const now = services.env.now();
  return (
    <>
      <OfflineNotices
        machines={machines}
        workspaceId={workspaceId}
        elsewhere={(machine) =>
          elsewhereText(machine, machines, summary, elsewhereCopy.newJobs)
        }
        now={now}
      />
      <FigureRow
        compact={compact}
        figures={[
          { label: listCopy.machines, value: figures.machines },
          { label: listCopy.slotsInUse, value: figures.slots },
          { label: listCopy.warmRunners, value: figures.warm },
          { label: listCopy.cacheHits, value: figures.cacheHits },
        ]}
      />
      <Panel title={listCopy.ownMachines} flush>
        <MachineTable
          machines={machines}
          workspaceId={workspaceId}
          now={now}
          viewport={viewport}
        />
      </Panel>
      <OverflowPanel
        snapshot={overflow}
        workspaceId={workspaceId}
        admin={admin}
        compact={compact}
        onRetry={onOverflowRetry}
      />
    </>
  );
}

const styles = StyleSheet.create({
  loading: { gap: 14, paddingVertical: 16, paddingHorizontal: 16 },
});
