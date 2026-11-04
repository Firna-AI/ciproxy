// The security section of the landing page: where code and secrets run, and
// the four guarantees behind it.

import type { LucideIcon } from "lucide-react";
import { Box, DatabaseZap, KeyRound, PlugZap } from "lucide-react";

import { Section, SectionHead } from "./section.tsx";

type Point = { icon: LucideIcon; title: string; text: string };

const points: readonly Point[] = [
  {
    icon: Box,
    title: "A fresh VM for every job",
    text: "Each job starts in a new VM, so it never reads the files or secrets of another job.",
  },
  {
    icon: PlugZap,
    title: "An agent that connects out",
    text: "Your machines open no inbound ports. The agent connects out to ciproxy.",
  },
  {
    icon: KeyRound,
    title: "Runner access only",
    text: "The GitHub App asks only to manage self-hosted runners and to list your repositories.",
  },
  {
    icon: DatabaseZap,
    title: "Trusted cache writes",
    text: "Only jobs on the default branch write to shared caches, so a pull request cannot poison them.",
  },
];

export function Security() {
  return (
    <Section id="security">
      <div className="security">
        <SectionHead
          eyebrow="Security"
          title="Your code stays in your accounts"
          lead="Jobs run on your machines and in your Boat account. Code and secrets go from GitHub straight to the runner; ciproxy decides only where the runner starts."
        />
        <ul className="security-points">
          {points.map((point) => (
            <li key={point.title} className="security-point">
              <point.icon size={18} strokeWidth={1.75} className="icon-muted" />
              <h3 className="security-title">{point.title}</h3>
              <p className="security-text">{point.text}</p>
            </li>
          ))}
        </ul>
      </div>
    </Section>
  );
}
