// The ciproxy mark and wordmark. The mark is two bars that meet as a chevron:
// one flow of jobs, routed to an own machine or to overflow. It takes the text
// colour, with no tile behind it.

import "./brand.css";

export function BrandMark({ size = 20 }: { size?: number }) {
  return (
    <svg
      aria-hidden="true"
      className="brand-mark"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="currentColor"
    >
      <path d="M2 3h8.6l8.4 8.1h-8.6z" />
      <path d="M10.4 12.9H19L10.6 21H2z" />
    </svg>
  );
}

export function Brand() {
  return (
    <span className="brand">
      <BrandMark />
      <span className="brand-word">ciproxy</span>
    </span>
  );
}
