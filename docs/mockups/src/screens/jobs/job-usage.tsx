// The CPU and memory panel of the job page. A finished job shows its average
// CPU and its peak memory against the VM that ran it, its out-of-memory kills
// on Linux, a chart over time with each step marked, and the figures of each
// step. The figures cover the whole VM while each step runs, so a service
// that an earlier step started counts in the later steps. The figures come
// once, when the job ends, so a running job says when they show. A job
// without figures, as from an older agent or image, says so in one sentence.
// A job whose steps ciproxy could not read shows its figures and the chart
// without step marks. When the VM that ran the job is larger than the size
// of its label, as a Boat size can be, each meter notes the label size with a
// tick, and each plot draws it as a line. A measured figure has one decimal,
// and a VM size or a label size reads as the Details panel writes it.

import type { ReactNode } from "react";
import type { Viewport } from "@mokly/mokly";
import { ProgressBar } from "@firna/ui/loader";
import { useSharedUiTheme } from "@firna/ui/theme";

import { Panel } from "../../components/page.tsx";
import { Dot } from "../../components/status.tsx";
import {
  dotJoin,
  formatDuration,
  formatGb,
  formatVcpu,
} from "../../fixtures/format.ts";
import type { JobUsage, MeasuredUsage } from "../../fixtures/job-usage.ts";
import { memoryText, sizePhrase, vcpuText } from "../../fixtures/sizes.ts";
import { UsageChart } from "./usage-chart.tsx";
import { processes, UsageSteps } from "./usage-steps.tsx";
import "./job-usage.css";

const title = "CPU and memory";

/** The size of the label on a meter: its text and its share of the VM. */
type LabelMark = { text: string; share: number };

/** A figure of the job, with a meter when it runs against the VM. */
function Figure({
  label,
  value,
  of,
  share,
  mark,
}: {
  label: string;
  value: ReactNode;
  /** The vCPU or the memory of the VM, such as "8 GB". */
  of?: string;
  /** The share of the VM that the figure uses. */
  share?: number;
  /** The size of the label, when the VM is larger. */
  mark?: LabelMark;
}) {
  const colors = useSharedUiTheme().colors;
  const vmText = of ? `of ${of}` : undefined;
  const caption =
    vmText && mark ? dotJoin(vmText, `label ${mark.text}`) : vmText;
  return (
    <div className="usage-figure">
      <span className="usage-figure-label">{label}</span>
      <span className="usage-figure-value">
        <span className="usage-figure-number">{value}</span>
        {caption ? <span className="caption">{caption}</span> : null}
      </span>
      {share === undefined ? null : (
        <span className="usage-meter">
          <ProgressBar
            value={Math.min(share, 1)}
            color={colors.primary}
            trackColor={colors.primarySoft}
            height={4}
            accessibilityLabel={label}
          />
          {mark ? (
            <span
              className="usage-meter-mark"
              style={{ left: `${(mark.share * 100).toFixed(2)}%` }}
              aria-hidden="true"
            />
          ) : null}
        </span>
      )}
    </div>
  );
}

function Measured({
  usage,
  viewport,
}: {
  usage: MeasuredUsage;
  viewport: Viewport;
}) {
  const { job, vm, labelSize } = usage;
  const vcpu = vcpuText(vm.vcpu);
  const memory = memoryText(vm.memoryGb);
  const cpuMark =
    labelSize && labelSize.vcpu < vm.vcpu
      ? { text: vcpuText(labelSize.vcpu), share: labelSize.vcpu / vm.vcpu }
      : undefined;
  const memoryMark =
    labelSize && labelSize.memoryGb < vm.memoryGb
      ? {
          text: memoryText(labelSize.memoryGb),
          share: labelSize.memoryGb / vm.memoryGb,
        }
      : undefined;
  const span = formatDuration(usage.cpu.length * usage.pointSeconds);
  const labelText = labelSize
    ? ` The label gives ${sizePhrase(labelSize)}.`
    : "";
  const summary = `CPU and memory over ${span}: average CPU ${formatVcpu(job.averageCpu)} of ${vcpu}, peak memory ${formatGb(job.peakMemoryGb)} of ${memory}.${labelText}`;
  return (
    <Panel title={title} flush>
      <div className="usage-body">
        <div className="usage-figures">
          <Figure
            label="Average CPU"
            value={formatVcpu(job.averageCpu)}
            of={vcpu}
            share={job.averageCpu / vm.vcpu}
            mark={cpuMark}
          />
          <Figure
            label="Peak memory"
            value={formatGb(job.peakMemoryGb)}
            of={memory}
            share={job.peakMemoryGb / vm.memoryGb}
            mark={memoryMark}
          />
          {job.oomKills === null ? null : (
            <Figure
              label="Out of memory"
              value={
                job.oomKills === 0 ? (
                  "None"
                ) : (
                  <span className="usage-figure-alert">
                    <Dot tone="err" />
                    <span>{processes(job.oomKills)}</span>
                  </span>
                )
              }
            />
          )}
        </div>
        <UsageChart usage={usage} summary={summary} viewport={viewport} />
      </div>
      {usage.steps ? (
        <>
          <UsageSteps steps={usage.steps} viewport={viewport} />
          <p className="caption usage-note">
            Each figure covers the whole VM, also the services and processes
            that earlier steps started.
          </p>
        </>
      ) : (
        <p className="small muted usage-note">
          Figures for each step are not available for this job.
        </p>
      )}
    </Panel>
  );
}

/** The panel of a job: its figures, or one sentence in their place. */
export function UsagePanel({
  usage,
  viewport,
}: {
  usage: JobUsage;
  viewport: Viewport;
}) {
  return (
    <div className="job-usage">
      {usage.state === "measured" ? (
        <Measured usage={usage} viewport={viewport} />
      ) : (
        <Panel title={title}>
          <p className="small muted">
            {usage.state === "running"
              ? "The figures show when the job ends."
              : "Figures are not available for this job."}
          </p>
        </Panel>
      )}
    </div>
  );
}
