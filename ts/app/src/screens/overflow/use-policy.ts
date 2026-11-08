// The overflow policy of Settings under Overflow: the switch, the wait, and
// the monthly limit. The API takes the complete policy, so each save sends
// the saved values with the one change. One save runs at a time, and the
// controls show the value on its way until the API answers. A failed save
// shows the saved value again, with what failed.

import { useCallback, useRef, useState } from "react";

import type { OverflowPolicy, OverflowSettings } from "../../api/schema.ts";
import type { OverflowStore } from "../../state/overflow.ts";
import {
  limitSaveProblem,
  policySaveFailure,
  type FieldProblem,
} from "../../view/boat-failures.ts";
import type { FailureCopy } from "../../view/claim.ts";

export type PolicySource = "wait" | "limit";

export type PolicyEditor = {
  /** The saved policy, or the one on its way to the API. */
  shown: OverflowPolicy;
  saving: PolicySource | null;
  waitFailure: FailureCopy | null;
  limitProblem: FieldProblem | null;
  save(source: PolicySource, change: Partial<OverflowPolicy>): Promise<void>;
};

export function usePolicy(
  store: OverflowStore,
  settings: OverflowSettings,
  limitCents: string,
): PolicyEditor {
  const { enabled, wait_seconds } = settings;
  const saved: OverflowPolicy = {
    enabled,
    wait_seconds,
    spend_limit_cents: limitCents,
  };
  const [pending, setPending] = useState<OverflowPolicy | null>(null);
  const [saving, setSaving] = useState<PolicySource | null>(null);
  const [waitFailure, setWaitFailure] = useState<FailureCopy | null>(null);
  const [limitProblem, setLimitProblem] = useState<FieldProblem | null>(null);
  const running = useRef(false);

  const save = useCallback(
    async (source: PolicySource, change: Partial<OverflowPolicy>) => {
      if (running.current) return;
      running.current = true;
      const policy: OverflowPolicy = {
        enabled,
        wait_seconds,
        spend_limit_cents: limitCents,
        ...change,
      };
      setPending(policy);
      setSaving(source);
      setWaitFailure(null);
      setLimitProblem(null);
      const result = await store.save(policy);
      running.current = false;
      setPending(null);
      setSaving(null);
      if (result.ok) return;
      if (source === "wait") setWaitFailure(policySaveFailure(result.failure));
      else setLimitProblem(limitSaveProblem(result.failure));
    },
    [enabled, limitCents, store, wait_seconds],
  );

  return {
    shown: pending ?? saved,
    saving,
    waitFailure,
    limitProblem,
    save,
  };
}
