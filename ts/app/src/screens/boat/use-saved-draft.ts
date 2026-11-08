// One field over a saved value, for both spend-limit forms: the Boat
// connection form and the limit panel of Settings under Overflow. The field
// shows the latest saved value until the person types, and the typed text
// stays until an accepted save of its own form. `state/saved-draft.ts` holds
// the rules.

import { useCallback, useState } from "react";

import { draftText, settleDraft, type Draft } from "../../state/saved-draft.ts";

export type SavedDraft = {
  /** The text of the field: the draft, else the latest saved value. */
  text: string;
  /** The person typed. The text is a draft, also when it equals a saved value. */
  edit(text: string): void;
  /** A save of this form ended. An accepted save ends the draft. */
  settle(accepted: boolean): void;
};

export function useSavedDraft(saved: string): SavedDraft {
  const [draft, setDraft] = useState<Draft>(null);
  const settle = useCallback((accepted: boolean) => {
    setDraft((current) => settleDraft(current, accepted));
  }, []);
  return { text: draftText(draft, saved), edit: setDraft, settle };
}
