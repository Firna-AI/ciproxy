// The privacy of a typed Boat key in the browser, for the Setup connection
// journey. The key field masks the key while it shows. No console line,
// browser storage, page URL, or page text holds a fixture key. A failed
// check names only the surface, never a key or the text of the surface.
import assert from "node:assert/strict";

import type { Page } from "./browser-driver.ts";
import { fixtureBoatKey, refusedBoatKey } from "./fixtures-boat.ts";

const fixtureKeys = [fixtureBoatKey, refusedBoatKey];

function holdsKey(text: string): boolean {
  return fixtureKeys.some((key) => text.includes(key));
}

/**
 * Count the console lines of the page that hold a fixture key, from now on.
 * The count keeps no line, so no failure can print one.
 */
export function watchKeyLines(page: Page): () => number {
  let lines = 0;
  page.on("console", (message) => {
    if (holdsKey(message.text())) lines += 1;
  });
  return () => lines;
}

/** Whether local storage, tab storage, or a script cookie holds a key. */
function storageHoldsKey(page: Page): Promise<boolean> {
  // No user-facing handle shows browser storage, so the page reads it.
  return page.evaluate(
    (keys) =>
      [window.localStorage, window.sessionStorage].some((storage) =>
        Array.from({ length: storage.length }, (_, index) => {
          const name = storage.key(index) ?? "";
          return `${name}=${storage.getItem(name) ?? ""}`;
        }).some((entry) => keys.some((key) => entry.includes(key))),
      ) || keys.some((key) => document.cookie.includes(key)),
    fixtureKeys,
  );
}

/** The names of the surfaces that hold a fixture key. */
async function keySurfaces(
  page: Page,
  keyLines: () => number,
): Promise<string[]> {
  const stored = await storageHoldsKey(page);
  const shown = await Promise.all(
    fixtureKeys.map((key) => page.getByText(key).count()),
  );
  // The fixture keys hold only letters and hyphens, which a URL never encodes.
  const surfaces: [string, boolean][] = [
    ["console", keyLines() > 0],
    ["browser storage", stored],
    ["page URL", holdsKey(page.url())],
    ["page text", shown.some((count) => count > 0)],
  ];
  return surfaces.filter(([, holds]) => holds).map(([name]) => name);
}

/** Check that no surface holds a fixture key at `phase`. */
export async function assertKeyPrivate(
  page: Page,
  keyLines: () => number,
  phase: string,
): Promise<void> {
  const surfaces = await keySurfaces(page, keyLines);
  assert.deepEqual(
    surfaces,
    [],
    `A fixture key shows in the ${surfaces.join(" and ")} ${phase}`,
  );
}

/** Check that the key field with this label masks its key. */
export async function assertKeyMasked(
  page: Page,
  label: string,
  phase: string,
): Promise<void> {
  assert.equal(
    await page.getByLabel(label, { exact: true }).getAttribute("type"),
    "password",
    `The key field shows its key in plain text ${phase}`,
  );
}
