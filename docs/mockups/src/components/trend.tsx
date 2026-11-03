// A short trend of recent values: a thin line, a faint fill below it for the
// area kind, and a dot at the latest value for the end kind. It stretches to
// its box, and the line keeps its width, so the stylesheet of the parent sets
// the size. The Firna Sparkline has no area fill and a fixed 2 px line, so
// the directions draw their own.

const width = 100;
const height = 32;
const inset = 2;

export function Trend({
  data,
  area = false,
  end = false,
}: {
  data: readonly number[];
  area?: boolean;
  end?: boolean;
}) {
  const low = Math.min(...data);
  const span = Math.max(...data) - low || 1;
  const step = data.length > 1 ? width / (data.length - 1) : width;
  const yOf = (value: number) =>
    inset + (1 - (value - low) / span) * (height - 2 * inset);
  const line = data
    .map((value, i) => {
      const x = (i * step).toFixed(2);
      return `${i === 0 ? "M" : "L"}${x} ${yOf(value).toFixed(2)}`;
    })
    .join("");
  const last = data[data.length - 1] ?? low;
  return (
    <span className="trend-box">
      <svg
        className="trend"
        viewBox={`0 0 ${width} ${height}`}
        preserveAspectRatio="none"
        aria-hidden="true"
      >
        {area ? (
          <path
            className="trend-area"
            d={`${line}L${width} ${height}L0 ${height}Z`}
          />
        ) : null}
        <path
          className="trend-line"
          d={line}
          vectorEffect="non-scaling-stroke"
        />
      </svg>
      {end ? (
        <span
          className="trend-end"
          style={{ top: `${((yOf(last) / height) * 100).toFixed(1)}%` }}
        />
      ) : null}
    </span>
  );
}
