// Copy to the clipboard. The hook keeps the result of the last press with
// the text that it wrote, so the control shows "Copied" or the failure only
// for the text that it offers now. A late answer of an earlier press changes
// nothing. The app writes the clipboard only when the person presses Copy.

import { useCallback, useRef, useState } from "react";
import { Button } from "@firna/ui/button";

import { copyState, type CopyResult, type CopyState } from "./copy-state.ts";
import { CopyIcon } from "./icons.tsx";

async function writeClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}

export function useCopy(text: string): { state: CopyState; copy: () => void } {
  const [result, setResult] = useState<CopyResult>(null);
  const press = useRef(0);
  const copy = useCallback(() => {
    press.current += 1;
    const mine = press.current;
    void writeClipboard(text).then((copied) => {
      if (mine === press.current) setResult({ text, copied });
    });
  }, [text]);
  return { state: copyState(result, text), copy };
}

/**
 * The Copy button. `subject` names what it copies, such as "the install
 * command", and completes its accessible name: "Copy the install command",
 * then "Copied the install command". `label` replaces the visible "Copy"
 * when the button stands alone, as "Copy prompt for your agent" does. The
 * label is then also the accessible name, so that a person can say the
 * words on the button.
 */
export function CopyButton({
  state,
  subject,
  label,
  onPress,
}: {
  state: CopyState;
  subject: string;
  label?: string;
  onPress: () => void;
}) {
  const copied = state === "copied";
  const verb = copied ? "Copied" : "Copy";
  const name = copied || label === undefined ? `${verb} ${subject}` : label;
  return (
    <Button
      size="sm"
      icon={CopyIcon}
      accessibilityLabel={name}
      onPress={onPress}
    >
      {copied ? "Copied" : (label ?? "Copy")}
    </Button>
  );
}
