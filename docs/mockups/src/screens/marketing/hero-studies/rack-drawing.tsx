import { useId } from "react";

import { machineScenes } from "../../../fixtures/machines.ts";

const machines = machineScenes.peak;

function Chassis({
  y,
  name,
  vents,
}: {
  y: number;
  name: string;
  vents: string;
}) {
  return (
    <g
      transform={`translate(0 ${y})`}
      stroke="var(--c-hero-cobalt)"
      strokeWidth="1.4"
      strokeLinejoin="round"
    >
      <g transform="matrix(1 .32 .85 -.52 44 163)">
        <rect width="222" height="171" fill="var(--c-hero-blueprint)" />
        <rect x="9" y="8" width="204" height="151" fill="none" />
        <rect
          x="18"
          y="60"
          width="150"
          height="88"
          fill={`url(#${vents})`}
          stroke="none"
        />
        <path d="M16 37h188M176 58v88m6-88v88m6-88v88" fill="none" />
      </g>
      <g transform="matrix(.85 -.52 0 1 266 234)">
        <rect width="171" height="30" fill="var(--c-hero-blueprint)" />
        {Array.from({ length: 24 }, (_, i) => (
          <path key={i} d={`M${10 + i * 6} 7v15`} strokeWidth="2.8" />
        ))}
      </g>
      <g transform="matrix(1 .32 0 1 44 163)">
        <rect width="222" height="30" fill="var(--c-surface)" />
        <rect x="5" y="6" width="5" height="17" fill="var(--c-hero-cobalt)" />
        <rect
          x="16"
          y="5"
          width="52"
          height="20"
          fill={`url(#${vents})`}
          stroke="none"
        />
        <text
          x="75"
          y="19"
          stroke="none"
          fill="var(--c-hero-cobalt)"
          fontFamily="var(--font-mono)"
          fontSize="10"
        >
          {name}
        </text>
        <path
          d="M180 11h17v9h-17zM201 11h10v9h-10z"
          fill="var(--c-hero-cobalt)"
        />
      </g>
    </g>
  );
}

export function RackDrawing() {
  const vents = useId();
  return (
    <svg className="rack-drawing" viewBox="0 0 460 380" aria-hidden="true">
      <defs>
        <pattern id={vents} width="5" height="5" patternUnits="userSpaceOnUse">
          <circle cx="2" cy="2" r="1.45" fill="var(--c-hero-cobalt)" />
        </pattern>
      </defs>
      {[...machines].reverse().map((machine, i) => (
        <Chassis
          key={machine.name}
          name={machine.name}
          vents={vents}
          y={(machines.length - 1 - i) * 46 - 18}
        />
      ))}
      <g transform="matrix(1 .32 .85 -.52 44 145)" fill="var(--c-hero-cobalt)">
        <text
          x="18"
          y="-15"
          transform="scale(1 -1)"
          fontFamily="var(--font-mono)"
          fontSize="19"
          fontWeight="700"
        >
          AX42
        </text>
      </g>
    </svg>
  );
}
