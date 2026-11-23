// The form of one provider connection. A provider whose key can bill one of
// several accounts, which today is Boat with its wallets, takes two phases:
// Check key first, then the wallet choice and Connect. The key check saves
// nothing and lists the wallets of the key, with the likely wallet picked.
// A key edit drops the wallets, and the answer of an older check never
// applies. A provider with no such choice connects in one step. Connect
// sends the generated request with the key, the limit, and the chosen
// wallet when there is one, and the form drops the key as soon as the API
// accepted it. One change runs at a time, and while another change of the
// workspace runs, the form waits for the pending mark of the store. Until
// the person types a limit, the field shows the latest saved limit of the
// provider, so a reconnect or a key change sends the newest saved limit. A
// typed limit stays until the API accepts the connection.

import { useCallback, useEffect, useRef, useState } from "react";

import type { ProviderSettings } from "../../api/schema.ts";
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
  walletMissing,
  walletOptions,
  type WalletOption,
} from "../../view/boat-wallets.ts";
import type { FailureCopy } from "../../view/claim.ts";
import { dollarsText, parseDollars } from "../../view/money.ts";
import {
  checkProblem,
  connectProblem,
  keyMissing,
  limitInvalid,
  type FieldProblem,
} from "../../view/overflow-failures.ts";
import { providerName } from "../../view/providers.ts";
import { useSavedDraft } from "./use-saved-draft.ts";

export type ConnectForm = {
  /** The provider that the form connects. */
  provider: ProviderSettings;
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
  /** Another change of the workspace runs, so the form waits. */
  locked: boolean;
  /** "Check key" before the wallets of the key, else "Connect Boat". */
  submitLabel: string;
  /** Check key before the check answered, and Connect after it. */
  submit(): Promise<void>;
};

export function useConnectForm(
  store: OverflowStore,
  provider: ProviderSettings,
  onConnected?: () => void,
): ConnectForm {
  const savedCents = provider.spend_limit_cents ?? null;
  const saved = savedCents === null ? "" : dollarsText(savedCents);
  const pending = useValue(store.pending);
  const [apiKey, setApiKey] = useState("");
  const {
    text: limitText,
    edit: editLimit,
    settle: settleLimit,
  } = useSavedDraft(saved);
  const [check, setCheck] = useState<KeyCheck>(uncheckedKey);
  const latest = useRef(check);
  const [keyError, setKeyError] = useState<string | null>(null);
  const [limitError, setLimitError] = useState<string | null>(null);
  const [walletError, setWalletError] = useState<string | null>(null);
  const [failure, setFailure] = useState<FailureCopy | null>(null);
  const [busy, setBusy] = useState(false);
  const running = useRef(false);
  const key = provider.provider;
  const choice = provider.has_billing_account_choice;

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
    async (typed: string) => {
      const started = nextAttempt(latest.current);
      update(started);
      const result = await store.checkKey(key, typed);
      if (latest.current.attempt !== started.attempt) return;
      if (result.ok)
        update(finishCheck(latest.current, started.attempt, result.value));
      else show(checkProblem(result.failure, key));
    },
    [key, show, store, update],
  );

  const connect = useCallback(
    async (typed: string, checked: CheckedKey | null) => {
      const amount = parseDollars(limitText);
      const noWallet = checked !== null && checked.chosen === null;
      setLimitError(amount.ok ? null : limitInvalid);
      setWalletError(noWallet ? walletMissing : null);
      if (!amount.ok || noWallet) return;
      const wallet = checked?.chosen ?? null;
      const result = await store.connect(
        key,
        {
          api_key: typed,
          spend_limit_cents: amount.cents,
          ...(wallet === null ? {} : { wallet }),
        },
        () => {
          setApiKey("");
          update(nextAttempt(latest.current));
        },
      );
      settleLimit(result.ok);
      if (result.ok) {
        onConnected?.();
        return;
      }
      const problem = connectProblem(result.failure, key);
      if (problem.field === "key") update(nextAttempt(latest.current));
      show(problem);
    },
    [key, limitText, onConnected, settleLimit, show, store, update],
  );

  const submit = useCallback(async () => {
    if (running.current) return;
    const typed = apiKey.trim();
    setKeyError(typed === "" ? keyMissing(key) : null);
    setFailure(null);
    if (typed === "") return;
    const checked = latest.current.checked;
    running.current = true;
    setBusy(true);
    try {
      if (choice && checked === null) await checkKey(typed);
      else await connect(typed, checked);
    } finally {
      running.current = false;
      setBusy(false);
    }
  }, [apiKey, checkKey, choice, connect, key]);

  return {
    provider,
    apiKey,
    setApiKey(value) {
      setApiKey(value);
      setKeyError(null);
      setWalletError(null);
      update(nextAttempt(latest.current));
    },
    limit: limitText,
    setLimit(value) {
      editLimit(value);
      setLimitError(null);
    },
    savedLimit: saved !== "" && limitText === saved,
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
    submitLabel:
      choice && check.checked === null
        ? "Check key"
        : `Connect ${providerName(key)}`,
    submit,
  };
}
