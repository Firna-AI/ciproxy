// Small browser memories that hold no secret: the end of the sign-in, the
// last workspace, and whether this tab started an install. Storage can be
// missing or refuse writes, for example in a private window, so every call
// falls back to memory and never throws.

export type KeyValueStorage = {
  get(key: string): string | null;
  set(key: string, value: string): void;
  remove(key: string): void;
};

export function memoryStorage(): KeyValueStorage {
  const values = new Map<string, string>();
  return {
    get: (key) => values.get(key) ?? null,
    set: (key, value) => {
      values.set(key, value);
    },
    remove: (key) => {
      values.delete(key);
    },
  };
}

/** Wrap a Web Storage area, or memory when the area is not available. */
export function webStorage(area: () => Storage | undefined): KeyValueStorage {
  const fallback = memoryStorage();
  const read = (): Storage | undefined => {
    try {
      return area();
    } catch {
      return undefined;
    }
  };
  return {
    get(key) {
      try {
        return read()?.getItem(key) ?? fallback.get(key);
      } catch {
        return fallback.get(key);
      }
    },
    set(key, value) {
      fallback.set(key, value);
      try {
        read()?.setItem(key, value);
      } catch {
        // The memory copy keeps the value for this document.
      }
    },
    remove(key) {
      fallback.remove(key);
      try {
        read()?.removeItem(key);
      } catch {
        // Nothing to remove when the area refuses access.
      }
    },
  };
}

export const storageKeys = {
  signInEnds: "ciproxy.sign-in-ends",
  lastWorkspace: "ciproxy.workspace",
  installStarted: "ciproxy.install-started",
} as const;
