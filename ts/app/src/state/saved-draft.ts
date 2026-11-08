// A form field over a saved value, such as the saved spend limit. While the
// person has not typed in the field, it shows the latest saved value, so no
// form keeps an old copy of a value that the person did not change. Typed
// text is a draft. The draft stays when the saved value changes, also when
// it equals a saved value or is empty, and after a failed save. Only an
// accepted save of the form itself ends the draft.

/** The typed text of a field, or null while the person has not typed. */
export type Draft = string | null;

/** The text of the field: the draft, else the latest saved value. */
export function draftText(draft: Draft, saved: string): string {
  return draft ?? saved;
}

/** The draft after a save of the form itself: an accepted save ends it. */
export function settleDraft(draft: Draft, accepted: boolean): Draft {
  return accepted ? null : draft;
}
