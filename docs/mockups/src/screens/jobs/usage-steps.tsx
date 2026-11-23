// The steps of a job with their figures: a table on desktop and a dense list
// on mobile, in the order of GitHub, from Set up job to Complete job. Each
// step leads with its number on GitHub. A step that shows as Run script, Pre
// Run script, or Post Run script links to that step on GitHub, where its log
// names what it ran. A skipped step, and a step that holds no point, such as
// a step of under a second, show no figures. The out-of-memory column shows
// only when a step ran out of memory.

import { ExternalLink } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { DataTable, type Column } from "../../components/data-table.tsx";
import { DotLabel } from "../../components/status.tsx";
import { formatDuration, formatGb, formatVcpu } from "../../fixtures/format.ts";
import type { UsageStep } from "../../fixtures/job-usage.ts";

/** "1 process", "2 processes": the processes that ran out of memory. */
export function processes(count: number): string {
  return count === 1 ? "1 process" : `${count} processes`;
}

const columns: readonly Column[] = [
  { key: "step", label: "Step", flex: 1 },
  { key: "duration", label: "Duration", width: 88, align: "right" },
  { key: "cpu", label: "Average CPU", width: 112, align: "right" },
  { key: "memory", label: "Peak memory", width: 112, align: "right" },
];

const oomColumn: Column = {
  key: "oom",
  label: "Out of memory",
  width: 128,
  align: "right",
};

/** The name of a step; a name that hides a script links to its step. */
function StepName({ step }: { step: UsageStep }) {
  if (!step.script) return <span className="step-name">{step.name}</span>;
  return (
    <span
      className="step-name step-script"
      title={`Open step ${step.number} on GitHub`}
    >
      <span className="text-link">{step.name}</span>
      <span className="sr-only">, opens step {step.number} on GitHub</span>
      <ExternalLink size={12} className="icon-muted" aria-hidden="true" />
    </span>
  );
}

function StepTitle({ step }: { step: UsageStep }) {
  return (
    <span className="step-title">
      <span className="step-number num">{step.number}</span>
      <StepName step={step} />
    </span>
  );
}

function Duration({ step }: { step: UsageStep }) {
  if (!step.span) return <span className="muted">Skipped</span>;
  return (
    <span className="duration">
      {formatDuration(step.span.end - step.span.start)}
    </span>
  );
}

/** A figure that a step does not have. */
function NoFigure() {
  return (
    <>
      <span className="muted" aria-hidden="true">
        —
      </span>
      <span className="sr-only">No figures</span>
    </>
  );
}

function cell(step: UsageStep, key: string) {
  const figures = step.figures;
  if (key === "step") return <StepTitle step={step} />;
  if (key === "duration") return <Duration step={step} />;
  if (!figures) return <NoFigure />;
  if (key === "cpu") return formatVcpu(figures.averageCpu);
  if (key === "memory") return formatGb(figures.peakMemoryGb);
  return figures.oomKills ? (
    <DotLabel tone="err">{processes(figures.oomKills)}</DotLabel>
  ) : null;
}

/** "1.9 vCPU average · 5.2 GB peak", the figures of a step on one line. */
function figureLine(step: UsageStep): string | null {
  if (!step.figures) return null;
  return `${formatVcpu(step.figures.averageCpu)} average · ${formatGb(step.figures.peakMemoryGb)} peak`;
}

function StepRows({ steps }: { steps: readonly UsageStep[] }) {
  return (
    <ul className="step-rows">
      {steps.map((step) => {
        const line = figureLine(step);
        const kills = step.figures?.oomKills ?? 0;
        return (
          <li key={step.number} className="step-row">
            <span className="step-row-main">
              <StepTitle step={step} />
              {line ? (
                <span className="caption step-figures">{line}</span>
              ) : null}
              {kills > 0 ? (
                <span className="small step-figures">
                  <DotLabel tone="err">
                    Out of memory: {processes(kills)}
                  </DotLabel>
                </span>
              ) : null}
            </span>
            <Duration step={step} />
          </li>
        );
      })}
    </ul>
  );
}

export function UsageSteps({
  steps,
  viewport,
}: {
  steps: readonly UsageStep[];
  viewport: Viewport;
}) {
  if (viewport === "mobile") return <StepRows steps={steps} />;
  const killed = steps.some((step) => (step.figures?.oomKills ?? 0) > 0);
  return (
    <div className="usage-steps">
      <DataTable
        columns={killed ? [...columns, oomColumn] : columns}
        rows={steps}
        rowKey={(step) => String(step.number)}
        cell={cell}
      />
    </div>
  );
}
