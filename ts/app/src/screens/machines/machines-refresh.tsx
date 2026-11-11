// The warning of a failed machines read. The list, the machine page, and Add
// a machine share one machines store, and each of them shows this warning
// from it. The page keeps the last machines that it read, with the install
// command and the baseline of Add. Try again reads the machines again, and
// the next good read, from Try again or the interval, clears the warning.

import { ActionButton } from "../../components/actions.tsx";
import { Notice } from "../../components/notice.tsx";
import { useValue } from "../../services/hooks.ts";
import type { MachinesStore } from "../../state/machines.ts";
import { listCopy } from "../../view/machines-copy.ts";

export function MachinesRefreshNotice({ store }: { store: MachinesStore }) {
  const snapshot = useValue(store.state);
  if (snapshot.status !== "ready" || snapshot.refreshFailure === null) {
    return null;
  }
  return (
    <Notice
      tone="warning"
      title={listCopy.refreshTitle}
      actions={
        <ActionButton
          label={listCopy.tryAgain}
          size="sm"
          onPress={() => void store.load()}
        />
      }
    >
      {listCopy.refreshBody}
    </Notice>
  );
}
