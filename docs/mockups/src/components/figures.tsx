// Small figures: the usage meter, the slots of a machine as a thin segmented
// bar, the code block with its Copy button, and a quantity the customer sets.

import type { ReactNode } from "react";
import { Copy, Minus, Plus } from "lucide-react";
import { Button } from "@firna/ui/button";
import { ProgressBar } from "@firna/ui/loader";
import { useSharedUiTheme } from "@firna/ui/theme";

import type { SlotState } from "../fixtures/machines.ts";
import { Action, noop } from "./actions.tsx";
import "./figures.css";

export type MeterTone = "normal" | "warning";

/** A ratio against a limit. The fill and the track share one ramp. */
export function Meter({
  label,
  used,
  max,
  value,
  detail,
  tone = "normal",
}: {
  label: string;
  used: number;
  max: number;
  /** The figure beside the label, such as "8,640 of 25,000". */
  value: ReactNode;
  detail?: ReactNode;
  tone?: MeterTone;
}) {
  const palette = useSharedUiTheme().colors;
  const colors =
    tone === "warning"
      ? { fill: palette.amber, track: palette.amberSoft }
      : { fill: palette.primary, track: palette.primarySoft };
  return (
    <div className="meter">
      <div className="spread">
        <span className="meter-label">{label}</span>
        <span className="meter-value num">{value}</span>
      </div>
      <ProgressBar
        value={Math.min(used / max, 1)}
        color={colors.fill}
        trackColor={colors.track}
        height={4}
        accessibilityLabel={label}
      />
      {detail ? <p className="caption">{detail}</p> : null}
    </div>
  );
}

const slotTitles: Record<SlotState, string> = {
  busy: "Running a job",
  warm: "Warm runner, ready",
  free: "Free",
};

function slotSummary(slots: readonly SlotState[]): string {
  const busy = slots.filter((s) => s === "busy").length;
  const warm = slots.filter((s) => s === "warm").length;
  return `${busy} of ${slots.length} busy, ${warm} warm`;
}

/**
 * One segment per slot of a machine: busy, warm runner, or free. The large
 * size is for a picture of the product, such as the landing page hero.
 */
export function SlotStrip({
  slots,
  size = "sm",
}: {
  slots: readonly SlotState[];
  size?: "sm" | "lg";
}) {
  return (
    <span
      className={`slot-strip slot-strip-${size}`}
      role="img"
      aria-label={slotSummary(slots)}
    >
      {slots.map((slot, i) => (
        <span
          key={i}
          className={`slot slot-${slot}`}
          title={slotTitles[slot]}
        />
      ))}
    </span>
  );
}

export function SlotLegend() {
  return (
    <span className="slot-legend caption">
      <span className="row">
        <span className="slot slot-busy" /> Running a job
      </span>
      <span className="row">
        <span className="slot slot-warm" /> Warm runner
      </span>
      <span className="row">
        <span className="slot slot-free" /> Free
      </span>
    </span>
  );
}

/** A command or a file excerpt to copy. Lines starting with + or - are a diff. */
export function CodeBlock({
  label,
  lines,
}: {
  /** Where the code goes, such as "On the machine" or a file path. */
  label: string;
  lines: readonly string[];
}) {
  return (
    <div className="code-block">
      <div className="code-head">
        <span className="code-label">{label}</span>
        <Action size="sm" icon={Copy}>
          Copy
        </Action>
      </div>
      <pre>
        {lines.map((line, i) => (
          <span key={i} className={codeLineClass(line)}>
            {line}
          </span>
        ))}
      </pre>
    </div>
  );
}

function codeLineClass(line: string): string {
  if (line.startsWith("+")) return "code-add";
  if (line.startsWith("-")) return "code-remove";
  return "code-line";
}

/** A quantity the customer sets, such as an add-on or a warm runner count. */
export function Quantity({
  value,
  label,
  caption,
}: {
  value: number;
  label: string;
  caption: string;
}) {
  return (
    <div className="quantity">
      <div className="stack-tight">
        <span className="quantity-label">{label}</span>
        <span className="caption">{caption}</span>
      </div>
      <span className="quantity-control">
        <Button
          tone="secondary"
          size="sm"
          shape="square"
          icon={Minus}
          accessibilityLabel="Fewer"
          onPress={value > 0 ? noop : undefined}
        />
        <span className="quantity-value num">{value}</span>
        <Button
          tone="secondary"
          size="sm"
          shape="square"
          icon={Plus}
          accessibilityLabel="More"
          onPress={noop}
        />
      </span>
    </div>
  );
}
