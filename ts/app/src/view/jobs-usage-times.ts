// The time axis of the CPU and memory chart: the time from the first sample,
// at round times under both plots. Each label belongs to its time on the time
// scale. A label is centred on its time, as the mockup draws it, unless the
// panel would cut it off. Then it starts at the visible left edge or ends at
// the visible right edge, so it moves only as far as it must, and its time
// stays under it. The space between the times is the shortest that gives at
// most 4 spaces on a phone and 8 on a wider window and keeps the labels
// apart at the width of the axis. The shortest space is one second, so each
// chart names a time after 0:00.

/** Where a label lies: centred on its time, or at the visible edge. */
export type TimeAlign = "start" | "center" | "end";

/** A label of the time axis, at a share of the axis from its start. */
export type TimeTick = { text: string; share: number; align: TimeAlign };

/**
 * The time axis in pixels: its width, and the room past its start and its
 * end that the panel still shows.
 */
export type TimeAxis = { width: number; left: number; right: number };

/**
 * The size of the axis labels of the chart, in pixels. They use the
 * monospace face of the tokens, whose glyphs are 0.6 em wide, so the width
 * of a label follows from its length.
 */
export const axisFontSize = 11;

const glyphWidth = axisFontSize * 0.6;

/** The least room between two time labels, in pixels. */
const labelGap = 8;

/** The spaces between time ticks, in seconds, shortest first. */
const tickSpaces = [
  1, 2, 5, 10, 15, 30, 60, 120, 300, 600, 900, 1800, 3600, 7200, 10800, 21600,
  43200, 86400,
];

const day = 86_400;

function pad(value: number): string {
  return String(value).padStart(2, "0");
}

/** 372 -> "6:12", 3720 -> "1:02:00": the time from the first sample. */
function clockText(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const rest = pad(Math.floor(seconds % 60));
  return hours > 0 ? `${hours}:${pad(minutes)}:${rest}` : `${minutes}:${rest}`;
}

/** The pixels from the start of the axis to each end of a label. */
function extent(tick: TimeTick, axis: TimeAxis): [number, number] {
  const length = tick.text.length * glyphWidth;
  const end = axis.width + axis.right;
  if (tick.align === "start") return [-axis.left, length - axis.left];
  if (tick.align === "end") return [end - length, end];
  const middle = tick.share * axis.width;
  return [middle - length / 2, middle + length / 2];
}

/** A label centred on its time, unless the panel would cut it off. */
function tickAt(text: string, share: number, axis: TimeAxis): TimeTick {
  const [left, right] = extent({ text, share, align: "center" }, axis);
  const align =
    left < -axis.left
      ? "start"
      : right > axis.width + axis.right
        ? "end"
        : "center";
  return { text, share, align };
}

/** A label at each multiple of the space. */
function ticksEvery(
  seconds: number,
  space: number,
  axis: TimeAxis,
): TimeTick[] {
  return Array.from({ length: Math.floor(seconds / space) + 1 }, (_, index) =>
    tickAt(clockText(index * space), (index * space) / seconds, axis),
  );
}

/** Each label keeps the least room from the next one. */
function apart(ticks: readonly TimeTick[], axis: TimeAxis): boolean {
  return ticks.every((tick, index) => {
    const next = ticks[index + 1];
    return (
      next === undefined ||
      extent(next, axis)[0] - extent(tick, axis)[1] >= labelGap
    );
  });
}

/**
 * The time ticks of a chart of `seconds` on an axis. Each space names at
 * least one time after 0:00. When no space keeps the labels apart, the axis
 * takes the longest space, which gives the fewest labels. A chart of under a
 * second has only its start. Before the axis has a width, it has no labels.
 */
export function timeTicks(
  seconds: number,
  axis: TimeAxis,
  mobile: boolean,
): TimeTick[] {
  if (axis.width <= 0) return [];
  const most = mobile ? 4 : 8;
  const days = Array.from(
    { length: Math.max(0, Math.floor(seconds / day) - 1) },
    (_, index) => (index + 2) * day,
  );
  const choices = [...tickSpaces, ...days]
    .filter((space) => space <= seconds && seconds / space <= most)
    .map((space) => ticksEvery(seconds, space, axis));
  return (
    choices.find((ticks) => apart(ticks, axis)) ??
    choices.at(-1) ?? [tickAt(clockText(0), 0, axis)]
  );
}
