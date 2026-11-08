// The notices of the machine page: a drain, a lost contact, a machine
// without KVM, a machine without room for a slot, and a machine whose disk
// holds fewer jobs than its slots. Each names what happens to the jobs, and
// the fix when one exists; only an Admin gets the install command of a fix.
// After Clear cache, or a change that the machine gets when it connects
// again, a notice says so, because the page shows the effect later.

import { LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import {
  DiskLimitNotice,
  NoKvmNotice,
} from "../../components/install-command.tsx";
import { paths } from "../../components/paths.ts";
import { offlineFor, type Machine } from "../../fixtures/machines.ts";

/** The states of the machine page that lead with a notice. */
export type NoticeState =
  | "draining"
  | "offline"
  | "no-slots"
  | "disk-limit"
  | "clear-requested"
  | "saved-offline"
  | "no-room";

function DrainingNotice({ machine }: { machine: Machine }) {
  const finishing = machine.slots.filter((slot) => slot === "busy").length;
  return (
    <Notice tone="warning" title="Draining">
      {finishing} jobs are finishing. The machine takes no new jobs, and it
      keeps no warm runners.
    </Notice>
  );
}

export function MachineNotice({
  state,
  machine,
  admin,
}: {
  state: NoticeState;
  machine: Machine;
  admin: boolean;
}) {
  const fix = admin ? (
    <LinkAction to={paths.addMachine} size="sm">
      Show install command
    </LinkAction>
  ) : undefined;
  switch (state) {
    case "draining":
      return <DrainingNotice machine={machine} />;
    case "saved-offline":
      return (
        <>
          <Notice tone="info" title="Saved">
            {machine.name} gets the change when it connects again.
          </Notice>
          <DrainingNotice machine={machine} />
        </>
      );
    case "clear-requested":
      return (
        <Notice tone="info" title={`${machine.name} is clearing its cache`} />
      );
    case "no-room":
      return (
        <Notice tone="danger" title={`${machine.name} has no slots`}>
          Its threads and memory leave no room for a job after the host reserve.
          Use a machine with more threads and memory.
        </Notice>
      );
    case "offline":
      return (
        <Notice tone="danger" title={`${machine.name} is offline`}>
          ciproxy lost contact with it {offlineFor} ago. Check that the machine
          is on and can reach ciproxy.dev.
        </Notice>
      );
    case "no-slots":
      return <NoKvmNotice name={machine.name} actions={fix} />;
    case "disk-limit":
      return (
        <DiskLimitNotice name={machine.name} hardware={machine.hardware} />
      );
  }
}
