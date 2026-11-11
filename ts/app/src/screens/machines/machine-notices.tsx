// The header actions and the notices of the machine page. An Admin gets the
// actions of the state: Drain and Remove, Stop draining, or Remove machine.
// One change runs at a time, so the other actions wait while one runs. The
// notice of a machine without KVM gives an Admin the install command, which
// is the fix.

import { Link } from "expo-router";

import type { MachineView } from "../../api/schema.ts";
import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { Notice } from "../../components/notice.tsx";
import type { ChangeSnapshot } from "../../state/machines-changes.ts";
import { noticeCopy } from "../../view/machines-copy.ts";
import {
  headerActions,
  machineNotice,
  type HeaderCommand,
} from "../../view/machines-detail.ts";
import { outcomeNotice } from "../../view/machines-outcome.ts";
import { addMachineHref } from "../../view/machines-routes.ts";

function runningCommand(change: ChangeSnapshot): HeaderCommand | null {
  if (change.status !== "busy" || change.change.kind !== "command") return null;
  const { command } = change.change;
  return command === "clear-cache" ? null : command;
}

export function HeaderActions({
  machine,
  change,
  onCommand,
}: {
  machine: MachineView;
  change: ChangeSnapshot;
  onCommand: (command: HeaderCommand) => void;
}) {
  const running = runningCommand(change);
  const busy = change.status === "busy";
  return headerActions(machine).map((action) => (
    <ActionButton
      key={action.command}
      label={action.label}
      tone={action.danger ? "danger" : "secondary"}
      busy={running === action.command}
      disabled={busy && running !== action.command}
      onPress={() => onCommand(action.command)}
    />
  ));
}

export function MachineNotices({
  machine,
  workspaceId,
  admin,
  change,
  now,
}: {
  machine: MachineView;
  workspaceId: string;
  admin: boolean;
  change: ChangeSnapshot;
  now: number;
}) {
  const notice = machineNotice(machine, now);
  const outcome = outcomeNotice(change, machine.hostname);
  const fix = addMachineHref(workspaceId);
  return (
    <>
      {outcome ? (
        <Notice tone={outcome.tone} title={outcome.title}>
          {outcome.body ?? undefined}
        </Notice>
      ) : null}
      {notice ? (
        <Notice
          tone={notice.tone}
          title={notice.title}
          actions={
            notice.fix && admin ? (
              <Link href={fix} push asChild>
                <ActionLink
                  label={noticeCopy.showCommand}
                  href={fix}
                  size="sm"
                />
              </Link>
            ) : undefined
          }
        >
          {notice.body}
        </Notice>
      ) : null}
    </>
  );
}
