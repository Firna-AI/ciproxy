// The form of one Boat connection: the key and the monthly limit. It checks
// both before it sends, keeps a refused key with its field error, and drops
// the key as soon as the API accepted it. One connection runs at a time, so
// a second press cannot send the key twice. The limit starts with the saved
// limit, so a reconnect keeps it unless the person changes it.

import { useCallback, useRef, useState } from "react";

import type { OverflowStore } from "../../state/overflow.ts";
import {
  connectProblem,
  keyMissing,
  limitInvalid,
} from "../../view/boat-failures.ts";
import type { FailureCopy } from "../../view/claim.ts";
import { dollarsText, parseDollars } from "../../view/money.ts";

export type ConnectForm = {
  apiKey: string;
  setApiKey(value: string): void;
  limit: string;
  setLimit(value: string): void;
  /** The limit field holds the saved limit, unchanged. */
  savedLimit: boolean;
  keyError: string | null;
  limitError: string | null;
  failure: FailureCopy | null;
  busy: boolean;
  submit(): Promise<void>;
};

export function useConnectForm(
  store: OverflowStore,
  savedLimitCents: string | null,
  onConnected?: () => void,
): ConnectForm {
  const saved = savedLimitCents === null ? "" : dollarsText(savedLimitCents);
  const [apiKey, setApiKey] = useState("");
  const [limit, setLimit] = useState(saved);
  const [keyError, setKeyError] = useState<string | null>(null);
  const [limitError, setLimitError] = useState<string | null>(null);
  const [failure, setFailure] = useState<FailureCopy | null>(null);
  const [busy, setBusy] = useState(false);
  const running = useRef(false);

  const submit = useCallback(async () => {
    if (running.current) return;
    const key = apiKey.trim();
    const amount = parseDollars(limit);
    setKeyError(key === "" ? keyMissing : null);
    setLimitError(amount.ok ? null : limitInvalid);
    setFailure(null);
    if (key === "" || !amount.ok) return;
    running.current = true;
    setBusy(true);
    const result = await store.connect(
      { apiKey: key, limitCents: amount.cents },
      () => setApiKey(""),
    );
    running.current = false;
    setBusy(false);
    if (result.ok) {
      onConnected?.();
      return;
    }
    const problem = connectProblem(result.failure);
    if (problem.field === null) setFailure(problem.copy);
    else if (problem.field === "key") setKeyError(problem.message);
    else setLimitError(problem.message);
  }, [apiKey, limit, onConnected, store]);

  return {
    apiKey,
    setApiKey(value) {
      setApiKey(value);
      setKeyError(null);
    },
    limit,
    setLimit(value) {
      setLimit(value);
      setLimitError(null);
    },
    savedLimit: saved !== "" && limit === saved,
    keyError,
    limitError,
    failure,
    busy,
    submit,
  };
}
