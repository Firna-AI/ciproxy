// The workspace policy of Settings under Overflow: the switch and the wait,
// which hold for every provider. A save sends only its edit, and the store
// adds the latest confirmed values when its turn comes. While any overflow
// change of the workspace runs, the store holds its pending mark, and the
// controls stay locked. The edit on its way shows in its control until the
// API answers. A failed save shows the saved value again, with what failed.

import { useCallback, useRef, useState } from "react";

import type { OverflowPolicy, OverflowSettings } from "../../api/schema.ts";
import { useValue } from "../../services/hooks.ts";
import type { OverflowStore } from "../../state/overflow.ts";
import type { FailureCopy } from "../../view/claim.ts";
import { policySaveFailure, unsentSave } from "../../view/overflow-failures.ts";

export type PolicyEditor = {
  /** The saved policy, with the edit on its way to the API. */
  shown: OverflowPolicy;
  /** An overflow change of the workspace runs, so every control waits. */
  locked: boolean;
  failure: FailureCopy | null;
  /** Save an edit. True when the API accepted it. */
  save(edit: Partial<OverflowPolicy>): Promise<boolean>;
};

export function usePolicy(
  store: OverflowStore,
  settings: OverflowSettings,
): PolicyEditor {
  const locked = useValue(store.pending);
  const [edit, setEdit] = useState<Partial<OverflowPolicy> | null>(null);
  const [failure, setFailure] = useState<FailureCopy | null>(null);
  const running = useRef(false);

  const save = useCallback(
    async (change: Partial<OverflowPolicy>) => {
      if (running.current) return false;
      running.current = true;
      setEdit(change);
      setFailure(null);
      const result = await store.save(change);
      running.current = false;
      setEdit(null);
      if (result !== null && result.ok) return true;
      setFailure(
        result === null ? unsentSave : policySaveFailure(result.failure),
      );
      return false;
    },
    [store],
  );

  return {
    shown: {
      enabled: settings.enabled,
      wait_seconds: settings.wait_seconds,
      ...edit,
    },
    locked,
    failure,
    save,
  };
}
