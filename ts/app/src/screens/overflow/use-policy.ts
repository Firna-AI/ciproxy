// The overflow policy of Settings under Overflow: the switch, the wait, and
// the monthly limit. A save sends only its edit, and the store adds the
// latest confirmed values when its turn comes. While any Boat change of the
// workspace runs, the store holds its pending mark, and the controls stay
// locked. The edit on its way shows in its control until the API answers. A
// failed save shows the saved value again, with what failed.

import { useCallback, useRef, useState } from "react";

import type { OverflowPolicy, OverflowSettings } from "../../api/schema.ts";
import { useValue } from "../../services/hooks.ts";
import type { OverflowStore, PolicyEdit } from "../../state/overflow.ts";
import {
  limitSaveProblem,
  policySaveFailure,
  unsentSave,
  type FieldProblem,
} from "../../view/boat-failures.ts";
import type { FailureCopy } from "../../view/claim.ts";

export type PolicySource = "wait" | "limit";

export type PolicyEditor = {
  /** The saved policy, with the edit on its way to the API. */
  shown: OverflowPolicy;
  /** The control whose save runs, for its busy state. */
  saving: PolicySource | null;
  /** A Boat change of the workspace runs, so every control waits. */
  locked: boolean;
  waitFailure: FailureCopy | null;
  limitProblem: FieldProblem | null;
  /** Save an edit. True when the API accepted it. */
  save(source: PolicySource, edit: PolicyEdit): Promise<boolean>;
};

export function usePolicy(
  store: OverflowStore,
  settings: OverflowSettings,
  limitCents: string,
): PolicyEditor {
  const locked = useValue(store.pending);
  const [edit, setEdit] = useState<PolicyEdit | null>(null);
  const [saving, setSaving] = useState<PolicySource | null>(null);
  const [waitFailure, setWaitFailure] = useState<FailureCopy | null>(null);
  const [limitProblem, setLimitProblem] = useState<FieldProblem | null>(null);
  const running = useRef(false);

  const save = useCallback(
    async (source: PolicySource, change: PolicyEdit) => {
      if (running.current) return false;
      running.current = true;
      setEdit(change);
      setSaving(source);
      setWaitFailure(null);
      setLimitProblem(null);
      const result = await store.save(change);
      running.current = false;
      setEdit(null);
      setSaving(null);
      if (result !== null && result.ok) return true;
      if (source === "wait") {
        setWaitFailure(
          result === null ? unsentSave : policySaveFailure(result.failure),
        );
      } else {
        setLimitProblem(
          result === null
            ? { field: null, copy: unsentSave }
            : limitSaveProblem(result.failure),
        );
      }
      return false;
    },
    [store],
  );

  return {
    shown: {
      enabled: settings.enabled,
      wait_seconds: settings.wait_seconds,
      spend_limit_cents: limitCents,
      ...edit,
    },
    saving,
    locked,
    waitFailure,
    limitProblem,
    save,
  };
}
