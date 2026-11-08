// The result of a copy, bound to the text that it copied. A result shows
// only while the copy control still offers that same text, so a new install
// command or a later page never shows "Copied" for a copy that did not
// happen, also when a slow clipboard answers late.

export type CopyState = "idle" | "copied" | "failed";

/** The last copy of a control: the text that it wrote, and whether it did. */
export type CopyResult = { text: string; copied: boolean } | null;

export function copyState(result: CopyResult, text: string): CopyState {
  if (result === null || result.text !== text) return "idle";
  return result.copied ? "copied" : "failed";
}

/**
 * The text that a diff leaves: the lines without a `-` marker, with the `+`
 * marker removed and the common indent cut, so it pastes as the new lines.
 */
export function afterText(lines: readonly string[]): string {
  const kept = lines
    .filter((line) => !line.startsWith("-"))
    .map((line) => (line.startsWith("+") ? ` ${line.slice(1)}` : line));
  const indent = Math.min(
    ...kept
      .filter((line) => line.trim() !== "")
      .map((line) => line.length - line.trimStart().length),
  );
  const cut = Number.isFinite(indent) ? indent : 0;
  return kept.map((line) => line.slice(cut)).join("\n");
}
