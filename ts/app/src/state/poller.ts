// A repeated background read. It waits for each read to finish before it
// schedules the next one, pauses while the tab is hidden, and reads at once
// when the tab shows again. The task records its own failures in its store,
// where the screen shows them, so a failed read is never silent.

export type Timers = {
  /** Call `callback` after `delay` ms; returns the function that cancels it. */
  schedule(callback: () => void, delay: number): () => void;
};

export type Visibility = {
  hidden(): boolean;
  /** Call the listener when the tab shows or hides; returns the remover. */
  watch(listener: () => void): () => void;
};

export type Poller = {
  /** Start reading on the interval. The first read waits one interval. */
  start(): void;
  stop(): void;
};

/**
 * Poll `task` every `interval` milliseconds while it returns true. A false
 * result ends the polling, for example when Setup has nothing to wait for.
 */
export function createPoller(
  task: () => Promise<boolean>,
  interval: number,
  timers: Timers,
  visibility: Visibility,
): Poller {
  let running = false;
  let busy = false;
  let cancel: (() => void) | null = null;
  let unwatch: (() => void) | null = null;

  const clear = () => {
    cancel?.();
    cancel = null;
  };

  const schedule = () => {
    clear();
    if (!running || busy || visibility.hidden()) return;
    cancel = timers.schedule(() => {
      cancel = null;
      void run();
    }, interval);
  };

  const run = async () => {
    if (!running || busy) return;
    busy = true;
    const again = await task().finally(() => {
      busy = false;
    });
    if (!again) running = false;
    schedule();
  };

  return {
    start() {
      if (running) return;
      running = true;
      unwatch = visibility.watch(() => {
        if (!running) return;
        if (visibility.hidden()) clear();
        else if (!busy) void run();
      });
      schedule();
    },
    stop() {
      running = false;
      clear();
      unwatch?.();
      unwatch = null;
    },
  };
}
