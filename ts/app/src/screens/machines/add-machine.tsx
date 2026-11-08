// Add another own machine with the one install command of Setup. The page
// reads the machines first, and shows the command only after that read, so a
// machine that installs fast still shows as new. Each visit opens with a
// usable command; a revoke during the visit keeps its notice. Then it waits
// for a machine that joins, and shows what the agent reported. A machine
// without KVM gets the notice with the fix. A failed read after the baseline
// keeps the command, the baseline, and the last machines, and shows the
// shared refresh warning with Try again; the next good read, from Try again
// or the interval, clears it and creates no command. The Admin role alone adds
// a machine, so another role goes back to the list.

import { useEffect, useMemo } from "react";
import { Link } from "expo-router";
import { StyleSheet, Text, View } from "react-native";

import { colors, radii, statusColors } from "@ciproxy/tokens";

import type { MachineView } from "../../api/schema.ts";
import { ActionLink } from "../../components/actions.tsx";
import { FeatureHeader, Panel } from "../../components/feature-parts.tsx";
import { DoneIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import { ReadFailure, WaitingLine } from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import { useServices } from "../../services/context.tsx";
import { useRedirect, useValue } from "../../services/hooks.ts";
import {
  addPollInterval,
  createAddTracker,
  joinedMachines,
} from "../../state/machines-add.ts";
import { machinesServices } from "../../state/machines-services.ts";
import {
  addCopy,
  detailCopy,
  listCopy,
  noticeCopy,
  slotCopy,
} from "../../view/machines-copy.ts";
import { machinesHref } from "../../view/machines-routes.ts";
import { concurrentJobs, connectedFacts } from "../../view/machines.ts";
import { InstallCommandView } from "../setup/install-command.tsx";
import type { WorkspaceContext } from "../workspace-page.tsx";
import { MachinesRefreshNotice } from "./machines-refresh.tsx";

function Joined({ machine }: { machine: MachineView }) {
  if (!machine.kvm) {
    return (
      <Notice tone="danger" title={noticeCopy.noKvmTitle(machine.hostname)}>
        {noticeCopy.noKvm}
      </Notice>
    );
  }
  const jobs = concurrentJobs(machine);
  return (
    <View style={styles.joined} role="status">
      <DoneIcon size={18} color={statusColors.ok} />
      <View style={styles.joinedText}>
        <Text style={[text.body, text.strong]}>
          {addCopy.connected(machine.hostname)}
        </Text>
        <Text style={text.caption}>{connectedFacts(machine)}</Text>
        {jobs === null ? null : (
          <Text style={text.caption}>{slotCopy.fits(jobs)}</Text>
        )}
      </View>
    </View>
  );
}

function AddBody({ context }: { context: WorkspaceContext }) {
  const services = useServices();
  const workspaceId = context.workspace.id;
  const store = machinesServices(services).machines(workspaceId);
  const tracker = useMemo(() => createAddTracker(store), [store]);
  const baseline = useValue(tracker.baseline);
  const failed = useValue(tracker.failed);
  const snapshot = useValue(store.state);
  useEffect(() => {
    const stop = tracker.start();
    const unwatch = store.watch(addPollInterval);
    return () => {
      stop();
      unwatch();
    };
  }, [tracker, store]);

  const ready = baseline !== null;
  useEffect(() => {
    if (!ready) return;
    // Each visit opens with a usable command. The store keeps an open command
    // until it ends, and creates one for a revoked, failed, or ended one.
    const command = services.installCommand(workspaceId);
    if (command.state.get().status !== "none") void command.create();
  }, [ready, services, workspaceId]);

  if (baseline === null) {
    return failed ? (
      <ReadFailure
        subject={listCopy.subject}
        onRetry={() => void tracker.retry()}
      />
    ) : (
      <WaitingLine>{listCopy.loading}</WaitingLine>
    );
  }
  const joined = joinedMachines(snapshot, baseline);
  const list = machinesHref(workspaceId);
  return (
    <>
      <Text style={text.small}>{addCopy.lead(context.workspace.name)}</Text>
      <InstallCommandView workspaceId={workspaceId} />
      <MachinesRefreshNotice store={store} />
      {joined.map((machine) => (
        <Joined key={machine.id} machine={machine} />
      ))}
      {joined.length > 0 ? (
        <View style={styles.row}>
          <Link href={list} push asChild>
            <ActionLink
              label={listCopy.openMachines}
              href={list}
              tone="primary"
            />
          </Link>
        </View>
      ) : (
        <WaitingLine>{addCopy.waiting}</WaitingLine>
      )}
    </>
  );
}

export function AddMachine({ context }: { context: WorkspaceContext }) {
  const admin = context.role === "admin";
  const list = machinesHref(context.workspace.id);
  useRedirect(admin ? null : list);
  return (
    <>
      <FeatureHeader
        crumb={{ label: detailCopy.crumb, href: list }}
        title={addCopy.title}
      />
      <Panel>
        <View style={styles.body}>
          {admin ? <AddBody context={context} /> : null}
        </View>
      </Panel>
    </>
  );
}

const styles = StyleSheet.create({
  body: { gap: 12, maxWidth: 720 },
  row: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
  joined: {
    flexDirection: "row",
    alignItems: "flex-start",
    gap: 10,
    paddingVertical: 12,
    paddingHorizontal: 14,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.lg,
    backgroundColor: colors.surface,
  },
  joinedText: { flex: 1, gap: 2 },
});
