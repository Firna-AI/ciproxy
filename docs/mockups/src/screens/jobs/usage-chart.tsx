// The CPU and memory of a job over time: two plots on one time axis, CPU
// above memory, because the two figures have different units and one plot
// with two value axes would invent a likeness between them. The top of each
// plot is the vCPU or the memory of the VM that ran the job. Each title names
// the unit of its plot, and its ticks read as the VM size does. A lane above
// the plots marks each step, with its number where it fits, and a hairline
// marks the start of each step in both plots. A red dot marks an
// out-of-memory kill. When the VM is larger than the size of the label, as a
// Boat size can be, a grey line marks the label size on each plot where it
// is smaller. The key under the plots names the line and the dot. The figure
// names the figures of the job for screen readers, and the step table under
// the chart gives the figures of each step.

import type { Viewport } from "@mokly/mokly";

import { dotJoin } from "../../fixtures/format.ts";
import type { MeasuredUsage, UsageStep } from "../../fixtures/job-usage.ts";
import { memoryText, vcpuText } from "../../fixtures/sizes.ts";
import "./usage-chart.css";

type Plot = {
  title: string;
  values: readonly number[];
  /** The top of the plot, the vCPU or the memory of the VM. */
  max: number;
  /** The unit of the plot, which its title names. */
  unit: string;
  /** The vCPU or the memory of the label, when it is smaller than the VM. */
  label: number | null;
};

/** The spaces between time ticks, in seconds, shortest first. */
const tickSpaces = [5, 10, 15, 30, 60, 120, 300, 600, 900, 1800, 3600];

/** The width that a lane gives its steps in the narrowest frame, in px. */
const laneWidth: Readonly<Record<Viewport, number>> = {
  desktop: 520,
  mobile: 270,
};

function percent(share: number): string {
  return `${(share * 100).toFixed(3)}%`;
}

/** 372 -> "6:12", the minutes and seconds from the start of the job. */
function clock(seconds: number): string {
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
}

/** The time ticks of a job: at most 8 spaces on desktop and 4 on mobile. */
function timeTicks(total: number, viewport: Viewport): number[] {
  const most = viewport === "mobile" ? 4 : 8;
  const space = tickSpaces.find((s) => total / s <= most) ?? 3600;
  return Array.from(
    { length: Math.floor(total / space) + 1 },
    (_, i) => i * space,
  );
}

/** The line and the area of a plot, in seconds across and 0 to 100 down. */
function shapes(plot: Plot, pointSeconds: number, total: number) {
  const ys = plot.values.map((v) => (100 - (v / plot.max) * 100).toFixed(2));
  const xs = plot.values.map((_, i) => (i + 0.5) * pointSeconds);
  const run = ys.map((y, i) => `L${xs[i]},${y}`).join("");
  const line = `M0,${ys[0]}${run}L${total},${ys.at(-1)}`;
  return {
    line,
    area: `M0,100L0,${ys[0]}${run}L${total},${ys.at(-1)}L${total},100Z`,
  };
}

/** The step numbers of a lane, where a number fits its block. */
function Lane({
  steps,
  total,
  viewport,
}: {
  steps: readonly UsageStep[];
  total: number;
  viewport: Viewport;
}) {
  return (
    <>
      <span className="uc-axis uc-lane-label">Step</span>
      <div className="uc-lane" aria-hidden="true">
        {steps.map((step) => {
          if (!step.span || step.span.end <= step.span.start) return null;
          const share = (step.span.end - step.span.start) / total;
          const fits = share * laneWidth[viewport] >= 22;
          return (
            <span
              key={step.number}
              className="uc-step"
              style={{
                left: percent(step.span.start / total),
                width: percent(share),
              }}
            >
              <span className="uc-step-block">{fits ? step.number : ""}</span>
            </span>
          );
        })}
      </div>
    </>
  );
}

function PlotRow({
  plot,
  usage,
  total,
  starts,
  kills,
}: {
  plot: Plot;
  usage: MeasuredUsage;
  total: number;
  starts: readonly number[];
  kills: readonly number[];
}) {
  const { line, area } = shapes(plot, usage.pointSeconds, total);
  const ticks = [plot.max, plot.max / 2, 0];
  const labelY =
    plot.label === null ? null : 100 - (plot.label / plot.max) * 100;
  return (
    <>
      <span className="uc-title">
        {plot.title} <span className="uc-unit">in {plot.unit}</span>
      </span>
      <div className="uc-y" aria-hidden="true">
        {ticks.map((tick) => (
          <span
            key={tick}
            className="uc-axis"
            style={{ bottom: percent(tick / plot.max) }}
          >
            {tick}
          </span>
        ))}
      </div>
      <div className="uc-plot">
        {ticks.map((tick) => (
          <span
            key={tick}
            className={`uc-grid${tick === 0 ? " is-base" : ""}`}
            style={{ bottom: percent(tick / plot.max) }}
          />
        ))}
        <svg
          viewBox={`0 0 ${total} 100`}
          preserveAspectRatio="none"
          aria-hidden="true"
        >
          {starts.map((start) => (
            <line
              key={start}
              className="uc-start"
              x1={start}
              x2={start}
              y1={0}
              y2={100}
            />
          ))}
          <path className="uc-area" d={area} />
          {labelY === null ? null : (
            <line
              className="uc-label"
              x1={0}
              x2={total}
              y1={labelY}
              y2={labelY}
            />
          )}
          <path className="uc-line" d={line} />
        </svg>
        {kills.map((i) => (
          <span
            key={i}
            className="uc-kill"
            style={{
              left: percent(((i + 0.5) * usage.pointSeconds) / total),
              top: percent(1 - (plot.values[i] ?? 0) / plot.max),
            }}
          />
        ))}
      </div>
    </>
  );
}

export function UsageChart({
  usage,
  summary,
  viewport,
}: {
  usage: MeasuredUsage;
  /** The figures of the job in words, for screen readers. */
  summary: string;
  viewport: Viewport;
}) {
  const total = usage.cpu.length * usage.pointSeconds;
  const starts = [
    ...new Set(
      (usage.steps ?? []).flatMap((step) =>
        step.span && step.span.start > 0 ? [step.span.start] : [],
      ),
    ),
  ];
  const kills = (usage.oomKills ?? []).flatMap((count, i) =>
    count > 0 ? [i] : [],
  );
  const { vm, labelSize } = usage;
  const cpuLabel =
    labelSize && labelSize.vcpu < vm.vcpu ? labelSize.vcpu : null;
  const memoryLabel =
    labelSize && labelSize.memoryGb < vm.memoryGb ? labelSize.memoryGb : null;
  const labelKey = [
    cpuLabel === null ? null : vcpuText(cpuLabel),
    memoryLabel === null ? null : memoryText(memoryLabel),
  ].filter((part) => part !== null);
  return (
    <figure className="uc" aria-label={summary}>
      {usage.steps ? (
        <Lane steps={usage.steps} total={total} viewport={viewport} />
      ) : null}
      <PlotRow
        plot={{
          title: "CPU",
          values: usage.cpu,
          max: vm.vcpu,
          unit: "vCPU",
          label: cpuLabel,
        }}
        usage={usage}
        total={total}
        starts={starts}
        kills={[]}
      />
      <PlotRow
        plot={{
          title: "Memory",
          values: usage.memoryGb,
          max: vm.memoryGb,
          unit: "GB",
          label: memoryLabel,
        }}
        usage={usage}
        total={total}
        starts={starts}
        kills={kills}
      />
      <div className="uc-x" aria-hidden="true">
        {timeTicks(total, viewport).map((tick) => (
          <span
            key={tick}
            className="uc-axis"
            style={{ left: percent(tick / total) }}
          >
            {clock(tick)}
          </span>
        ))}
      </div>
      {kills.length > 0 || labelKey.length > 0 ? (
        <figcaption className="uc-keys">
          {labelKey.length > 0 ? (
            <span className="uc-key">
              <span className="uc-key-label" aria-hidden="true" />
              Label size: {dotJoin(...labelKey)}
            </span>
          ) : null}
          {kills.length > 0 ? (
            <span className="uc-key">
              <span className="uc-key-kill" aria-hidden="true" />
              Out of memory
            </span>
          ) : null}
        </figcaption>
      ) : null}
    </figure>
  );
}
