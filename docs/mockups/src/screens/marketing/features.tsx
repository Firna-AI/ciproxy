// What ciproxy does, in six cells: own machines first, overflow on demand, the
// transparent cache, the savings report, security, and the open-source agent.

import type { LucideIcon } from "lucide-react";
import {
  Cloud,
  Github,
  HardDrive,
  PiggyBank,
  Server,
  ShieldCheck,
} from "lucide-react";

import { Cells, Section, SectionHead } from "./section.tsx";

type Feature = { icon: LucideIcon; title: string; text: string };

const features: readonly Feature[] = [
  {
    icon: Server,
    title: "Own machines first",
    text: "Jobs start at once on warm runners on your machines, each in a fresh VM.",
  },
  {
    icon: Cloud,
    title: "Overflow on demand",
    text: "When every slot is busy, ciproxy starts runners in your Boat account, billed by the second, up to the limit you set.",
  },
  {
    icon: HardDrive,
    title: "Transparent cache",
    text: "actions/cache reads and writes on the local disk of the machine. Your workflows stay as they are.",
  },
  {
    icon: PiggyBank,
    title: "Savings report",
    text: "See what your jobs cost each day, next to the same jobs on GitHub-hosted runners and Blacksmith.",
  },
  {
    icon: ShieldCheck,
    title: "Secure by default",
    text: "A fresh VM for every job, an agent that only connects out, and a GitHub App with runner access only.",
  },
  {
    icon: Github,
    title: "Open-source agent",
    text: "Read and audit the code that runs on your machines. The agent is open source under Apache-2.0.",
  },
];

export function Features() {
  return (
    <Section flush>
      <SectionHead
        eyebrow="Features"
        title={["Your machines first.", "The cloud when they are full."]}
        lead="ciproxy decides where each runner starts, so your hardware stays busy and no job waits for long."
      />
      <Cells columns={3} className="feature-cells">
        {features.map((feature) => (
          <div key={feature.title} className="mk-cell feature">
            <span className="feature-icon" aria-hidden="true">
              <feature.icon size={18} strokeWidth={1.75} />
            </span>
            <h3 className="feature-title">{feature.title}</h3>
            <p className="feature-text">{feature.text}</p>
          </div>
        ))}
      </Cells>
    </Section>
  );
}
