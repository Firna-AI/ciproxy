// The notices of the machine page: a drain, a lost contact, a machine
// without KVM, and a machine whose disk holds fewer jobs than its slots.
// Each names what happens to the jobs, and the fix when one exists; only an
// Admin gets the install command of a fix.

import { LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import {
  DiskLimitNotice,
  NoKvmNotice,
} from "../../components/install-command.tsx";
import { paths } from "../../components/paths.ts";
import { offlineFor, type Machine } from "../../fixtures/machines.ts";

/** The states of the machine page that lead with a notice. */
export type NoticeState = "draining" | "offline" | "no-slots" | "disk-limit";

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
    case "draining": {
      const finishing = machine.slots.filter((slot) => slot === "busy").length;
      return (
        <Notice tone="warning" title="Draining">
          {finishing} jobs are finishing. The machine takes no new jobs, and it
          keeps no warm runners.
        </Notice>
      );
    }
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
