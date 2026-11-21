// The first listener of the browser history. For `popstate` on the window,
// the browser calls the listeners in the order in which they were added,
// also a capture listener. The router adds its listener when it mounts. This
// module adds its listener when the routes load, which is earlier, so the
// navigation gate sees a history move first and can stop the router from
// opening the page in place after an API build change.

export type HistoryListener = (event: {
  stopImmediatePropagation(): void;
}) => void;

const listeners = new Set<HistoryListener>();

if (typeof window !== "undefined") {
  window.addEventListener(
    "popstate",
    (event) => {
      for (const listener of [...listeners]) listener(event);
    },
    true,
  );
}

export const earlyHistory = {
  add(listener: HistoryListener): void {
    listeners.add(listener);
  },
  remove(listener: HistoryListener): void {
    listeners.delete(listener);
  },
};
