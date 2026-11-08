// The live fit of the label dialog. While the person types a size, the
// dialog asks the API how many jobs of it each machine runs. A short pause
// after the last change sends one request, and only the answer to the latest
// request shows, so a slow answer never replaces a newer one. The fit keeps
// the label that it answers, so its line never mixes two drafts.

import type { ApiFailure, ApiResult } from "../api/client.ts";
import type { LabelFit, LabelInput } from "../api/schema.ts";
import type { Timers } from "./poller.ts";
import { createStore, type Store } from "./store.ts";

export type FitPreview = { input: LabelInput; fit: LabelFit };

export type PreviewSnapshot = {
  /** The fit of the latest answered draft, kept while a newer one waits. */
  shown: FitPreview | null;
  /** A newer draft waits for its answer. */
  pending: boolean;
  /** The latest request failed. */
  failure: ApiFailure | null;
};

export type LabelPreview = {
  readonly state: Store<PreviewSnapshot>;
  /** Ask for the fit of a draft after the pause; null clears a waiting ask. */
  request(input: LabelInput | null): void;
  /** Forget waiting asks and answers, for a closed dialog. */
  stop(): void;
};

export const previewDelay = 300;

export function createLabelPreview(options: {
  preview: (input: LabelInput) => Promise<ApiResult<LabelFit>>;
  timers: Timers;
  /** The fit that the API gave with the saved label, shown at first. */
  initial: FitPreview | null;
}): LabelPreview {
  const state = createStore<PreviewSnapshot>({
    shown: options.initial,
    pending: false,
    failure: null,
  });
  let latest = 0;
  let cancel: (() => void) | null = null;

  const clear = (): void => {
    cancel?.();
    cancel = null;
  };

  const send = async (input: LabelInput, sequence: number): Promise<void> => {
    const result = await options.preview(input);
    if (sequence !== latest) return;
    state.set(
      result.ok
        ? { shown: { input, fit: result.value }, pending: false, failure: null }
        : { ...state.get(), pending: false, failure: result.failure },
    );
  };

  return {
    state,
    request(input) {
      clear();
      latest += 1;
      const sequence = latest;
      if (input === null) {
        state.set({ ...state.get(), pending: false, failure: null });
        return;
      }
      state.set({ ...state.get(), pending: true });
      cancel = options.timers.schedule(() => {
        cancel = null;
        void send(input, sequence);
      }, previewDelay);
    },
    stop() {
      clear();
      latest += 1;
    },
  };
}
