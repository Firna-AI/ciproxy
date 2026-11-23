// The backgrounds of the landing hero. The landing page uses the grid. The
// other backdrops are the variations of marketing/hero-backgrounds, from which
// the look of the hero gets chosen. Each picture comes from the fixtures: the
// workflow file of mokly-ai and its starts of the last 30 days.

import { Fragment } from "react";
import type { Viewport } from "@mokly/mokly";

import { dailyStarts } from "../../fixtures/savings.ts";
import { heroWorkflow } from "../../fixtures/workflow.ts";
import "./hero-backdrops.css";

export type HeroBackdrop =
  "grid" | "plain" | "ink" | "workflow" | "starts" | "rule";

/**
 * The picture layer of a backdrop, behind the hero copy. The grid and the
 * plain backdrops need none.
 */
export function HeroBackdropArt({
  backdrop,
  viewport,
}: {
  backdrop: HeroBackdrop;
  viewport: Viewport;
}) {
  switch (backdrop) {
    case "ink":
      return <div className="hb-layer hb-ink" aria-hidden="true" />;
    case "workflow":
      return viewport === "desktop" ? <WorkflowArt /> : null;
    case "starts":
      return <StartsArt />;
    case "rule":
      return <RuleArt />;
    default:
      return null;
  }
}

/** The line where the workflow wraps from the left to the right column. */
const workflowSplit = 14;

function WorkflowArt() {
  const columns = [
    heroWorkflow.slice(0, workflowSplit),
    heroWorkflow.slice(workflowSplit),
  ];
  return (
    <div className="hb-layer hb-workflow" aria-hidden="true">
      {columns.map((lines, column) => (
        <ol key={column} className="hb-code" start={column * workflowSplit + 1}>
          {lines.map((line, index) => (
            <li
              key={index}
              className={line.includes("runs-on:") ? "is-change" : undefined}
            >
              {line || " "}
            </li>
          ))}
        </ol>
      ))}
    </div>
  );
}

/** The starts of each day of the report, own machines and Boat. */
const startDays = dailyStarts.labels.map((label, i) => ({
  label,
  own: dailyStarts.own[i] ?? 0,
  boat: dailyStarts.boat[i] ?? 0,
}));

function StartsArt() {
  const peak = Math.max(...startDays.map((day) => day.own + day.boat));
  const share = (value: number) => `${(value / peak) * 100}%`;
  return (
    <div className="hb-layer hb-starts" aria-hidden="true">
      <div className="hb-bars">
        {startDays.map((day) => (
          <span key={day.label} className="hb-day">
            <span className="hb-boat" style={{ height: share(day.boat) }} />
            <span className="hb-own" style={{ height: share(day.own) }} />
          </span>
        ))}
      </div>
    </div>
  );
}

function RuleArt() {
  return (
    <div className="hb-layer hb-rule" aria-hidden="true">
      {["is-top", "is-bottom"].map((edge) => (
        <Fragment key={edge}>
          <span className={`hb-line ${edge}`} />
          <span className={`hb-cross ${edge} is-left`} />
          <span className={`hb-cross ${edge} is-right`} />
        </Fragment>
      ))}
    </div>
  );
}
