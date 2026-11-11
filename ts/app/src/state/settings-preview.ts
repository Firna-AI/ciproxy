// The live fit of the label dialog. Each change of a draft enters through
// `request`, which first cancels the waiting pause and invalidates every
// ask in flight. Then it reuses the shown fit when that fit answers the same
// size, or asks the API after a short pause. Only the answer to the latest
// ask shows, so a slow answer never replaces a newer draft. The fit keeps the
// label that it answers, so its line never mixes two drafts.

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
  /**
   * The one entry for each change of the draft. It ends all earlier work,
   * then reuses the shown fit for the same size, or asks after the pause.
   * Null ends the earlier work and asks for nothing.
   */
  request(input: LabelInput | null): void;
  /** Forget waiting asks and answers, for a closed dialog. */
  stop(): void;
};

export const previewDelay = 300;

/**
 * The parts of a label that its fit depends on: the size, the switches "Your
 * machines" and "Boat", and "When nothing fits", which decide the Boat fit
 * and where the jobs run. A change of the name needs no new fit.
 */
export function fitKey(input: LabelInput): string {
  return [
    input.vcpus,
    input.memory_gb,
    input.disk_gb,
    input.allow_machines,
    input.allow_boat,
    input.when_nothing_fits,
  ].join("/");
}

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

  /** Cancel the waiting pause, and make every ask in flight obsolete. */
  const invalidate = (): number => {
    cancel?.();
    cancel = null;
    latest += 1;
    return latest;
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
      const sequence = invalidate();
      if (input === null) {
        state.set({ ...state.get(), pending: false, failure: null });
        return;
      }
      const { shown } = state.get();
      if (shown !== null && fitKey(shown.input) === fitKey(input)) {
        state.set({ shown, pending: false, failure: null });
        return;
      }
      state.set({ ...state.get(), pending: true });
      cancel = options.timers.schedule(() => {
        cancel = null;
        void send(input, sequence);
      }, previewDelay);
    },
    stop() {
      invalidate();
    },
  };
}
