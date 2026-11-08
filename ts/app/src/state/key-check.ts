// The key check of one Boat connection form. The form checks a key, shows
// the wallets of the API answer with its likely wallet picked, and connects
// the chosen wallet. Each check and each key edit starts a new attempt, so
// the answer of an older attempt never applies to another key. A form that
// is not checked holds no wallet at all, so no wallet value can stand for it.

import type { BoatWallet, BoatWallets } from "../api/schema.ts";

export type CheckedKey = {
  wallets: readonly BoatWallet[];
  /** The chosen wallet identity. Null when the API picked none of them. */
  chosen: string | null;
};

export type KeyCheck = { attempt: number; checked: CheckedKey | null };

export const uncheckedKey: KeyCheck = { attempt: 0, checked: null };

/** A new key or a new check: the wallets go, and older answers stop. */
export function nextAttempt(state: KeyCheck): KeyCheck {
  return { attempt: state.attempt + 1, checked: null };
}

/**
 * The answer of check `attempt`, which applies only while no key edit and
 * no newer check came after it. The likely wallet of the answer starts
 * chosen, when the answer lists it.
 */
export function finishCheck(
  state: KeyCheck,
  attempt: number,
  answer: BoatWallets,
): KeyCheck {
  if (attempt !== state.attempt) return state;
  const listed = answer.wallets.some(
    (item) => item.wallet === answer.selected_wallet,
  );
  return {
    attempt,
    checked: {
      wallets: answer.wallets,
      chosen: listed ? answer.selected_wallet : null,
    },
  };
}

/** Choose a wallet of the checked key. Any other value changes nothing. */
export function chooseWallet(state: KeyCheck, wallet: string): KeyCheck {
  const checked = state.checked;
  if (checked === null) return state;
  if (!checked.wallets.some((item) => item.wallet === wallet)) return state;
  return { ...state, checked: { ...checked, chosen: wallet } };
}
