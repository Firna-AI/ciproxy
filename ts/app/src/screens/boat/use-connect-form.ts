// The form of one Boat connection: Check key first, then the wallet choice
// and Connect Boat. The key check saves nothing and lists the wallets of the
// key, with the likely wallet picked. A key edit drops the wallets, and the
// answer of an older check never applies. Connect Boat sends the checked key,
// the limit, and the chosen wallet, and the form drops the key as soon as
// the API accepted it. One change runs at a time, and while another Boat
// change of the workspace runs, the form waits for the pending mark of the
// store. The limit starts with the saved limit, so a reconnect keeps it.

import { useCallback, useEffect, useRef, useState } from "react";

import { useValue } from "../../services/hooks.ts";
import {
  chooseWallet,
  finishCheck,
  nextAttempt,
  uncheckedKey,
  type CheckedKey,
  type KeyCheck,
} from "../../state/key-check.ts";
import type { OverflowStore } from "../../state/overflow.ts";
import {
  checkProblem,
  connectProblem,
  keyMissing,
  limitInvalid,
  walletMissing,
  type FieldProblem,
} from "../../view/boat-failures.ts";
import { walletOptions, type WalletOption } from "../../view/boat.ts";
import type { FailureCopy } from "../../view/claim.ts";
import { dollarsText, parseDollars } from "../../view/money.ts";

export type ConnectForm = {
  apiKey: string;
  setApiKey(value: string): void;
  limit: string;
  setLimit(value: string): void;
  /** The limit field holds the saved limit, unchanged. */
  savedLimit: boolean;
  /** The wallets of the checked key, or null before a check answered. */
  wallets: WalletOption[] | null;
  /** The chosen wallet identity, or null when none is chosen. */
  wallet: string | null;
  setWallet(value: string): void;
  keyError: string | null;
  limitError: string | null;
  walletError: string | null;
  failure: FailureCopy | null;
  /** This form's check or connection runs. */
  busy: boolean;
  /** Another Boat change of the workspace runs, so the form waits. */
  locked: boolean;
  /** Check key before the check answered, and Connect Boat after it. */
  submit(): Promise<void>;
};

export function useConnectForm(
  store: OverflowStore,
  savedLimitCents: string | null,
  onConnected?: () => void,
): ConnectForm {
  const saved = savedLimitCents === null ? "" : dollarsText(savedLimitCents);
  const pending = useValue(store.pending);
  const [apiKey, setApiKey] = useState("");
  const [limit, setLimit] = useState(saved);
  const [check, setCheck] = useState<KeyCheck>(uncheckedKey);
  const latest = useRef(check);
  const [keyError, setKeyError] = useState<string | null>(null);
  const [limitError, setLimitError] = useState<string | null>(null);
  const [walletError, setWalletError] = useState<string | null>(null);
  const [failure, setFailure] = useState<FailureCopy | null>(null);
  const [busy, setBusy] = useState(false);
  const running = useRef(false);

  const update = useCallback((next: KeyCheck) => {
    latest.current = next;
    setCheck(next);
  }, []);

  // A form that leaves its store takes no late answer of a check.
  useEffect(
    () => () => {
      latest.current = nextAttempt(latest.current);
    },
    [store],
  );

  const show = useCallback((problem: FieldProblem) => {
    if (problem.field === null) setFailure(problem.copy);
    else if (problem.field === "key") setKeyError(problem.message);
    else setLimitError(problem.message);
  }, []);

  const checkKey = useCallback(
    async (key: string) => {
      const started = nextAttempt(latest.current);
      update(started);
      const result = await store.checkKey(key);
      if (latest.current.attempt !== started.attempt) return;
      if (result.ok)
        update(finishCheck(latest.current, started.attempt, result.value));
      else show(checkProblem(result.failure));
    },
    [show, store, update],
  );

  const connect = useCallback(
    async (key: string, checked: CheckedKey) => {
      const amount = parseDollars(limit);
      setLimitError(amount.ok ? null : limitInvalid);
      setWalletError(checked.chosen === null ? walletMissing : null);
      if (!amount.ok || checked.chosen === null) return;
      const result = await store.connect(
        { apiKey: key, limitCents: amount.cents, wallet: checked.chosen },
        () => {
          setApiKey("");
          update(nextAttempt(latest.current));
        },
      );
      if (result.ok) {
        onConnected?.();
        return;
      }
      const problem = connectProblem(result.failure);
      if (problem.field === "key") update(nextAttempt(latest.current));
      show(problem);
    },
    [limit, onConnected, show, store, update],
  );

  const submit = useCallback(async () => {
    if (running.current) return;
    const key = apiKey.trim();
    setKeyError(key === "" ? keyMissing : null);
    setFailure(null);
    if (key === "") return;
    const checked = latest.current.checked;
    running.current = true;
    setBusy(true);
    try {
      if (checked === null) await checkKey(key);
      else await connect(key, checked);
    } finally {
      running.current = false;
      setBusy(false);
    }
  }, [apiKey, checkKey, connect]);

  return {
    apiKey,
    setApiKey(value) {
      setApiKey(value);
      setKeyError(null);
      setWalletError(null);
      update(nextAttempt(latest.current));
    },
    limit,
    setLimit(value) {
      setLimit(value);
      setLimitError(null);
    },
    savedLimit: saved !== "" && limit === saved,
    wallets:
      check.checked === null ? null : walletOptions(check.checked.wallets),
    wallet: check.checked?.chosen ?? null,
    setWallet(value) {
      update(chooseWallet(latest.current, value));
      setWalletError(null);
    },
    keyError,
    limitError,
    walletError,
    failure,
    busy,
    locked: pending && !busy,
    submit,
  };
}
