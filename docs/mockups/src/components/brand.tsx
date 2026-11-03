// The ciproxy mark and wordmark. The split arrow stands for the routing of a
// job to an own machine or to overflow. Inside a design direction, the mark
// is a geometric split chevron in the text colour, with no tile behind it.

import { Split } from "lucide-react";

import { useDirection } from "../theme/direction.ts";
import "./brand.css";

/** Two bars that meet as a chevron: one flow of jobs, routed two ways. */
function GeometricMark({ size }: { size: number }) {
  return (
    <svg
      aria-hidden="true"
      className="brand-geo"
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

export function BrandMark({ size = 26 }: { size?: number }) {
  const direction = useDirection();
  if (direction) return <GeometricMark size={Math.round(size * 0.75)} />;
  return (
    <span
      aria-hidden="true"
      className="brand-mark"
      style={{ width: size, height: size }}
    >
      <Split size={Math.round(size * 0.6)} strokeWidth={2.4} />
    </span>
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
