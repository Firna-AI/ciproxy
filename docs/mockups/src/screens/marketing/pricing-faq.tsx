// The common questions of the pricing page: what a start is, what counts as
// a machine, what happens past a limit, who pays for overflow, how to cancel,
// and how the landing page counts its saving. Every answer follows a decision
// or a design call of the plan.

import type { ReactNode } from "react";

import { TextLink } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";
import { formatCount } from "../../fixtures/format.ts";
import { tiers } from "../../fixtures/tiers.ts";
import {
  SavingCalculation,
  savingId,
  savingQuestion,
} from "./saving-calculation.tsx";
import { Section, SectionHead } from "./section.tsx";
import "./pricing-faq.css";

type Faq = { question: string; answer: ReactNode; id?: string };

const faqs: readonly Faq[] = [
  {
    question: "What is a start?",
    answer:
      "A start is one job that a ciproxy runner takes, on your machine or on Boat, whatever its result. A re-run is a new start. A warm runner that waits without a job is not a start.",
  },
  {
    question: "What counts as an own machine?",
    answer:
      "Every machine that is registered to your organization, also while it is offline. Remove a machine in the dashboard to free its place.",
  },
  {
    question: "What happens when I pass a limit?",
    answer: `Your jobs keep running. The dashboard shows a warning and offers the plan or add-on that covers your use, such as ${formatCount(tiers.team.starts)} starts on Team. You decide when to change; ciproxy adds no charges by itself.`,
  },
  {
    question: "Who pays for overflow?",
    answer:
      "You pay Boat directly. Overflow runners start in your own Boat account, which bills by the second. Set a monthly spend limit in ciproxy; at the limit, jobs wait for a free slot on your machines.",
  },
  {
    question: "How do I cancel?",
    answer:
      "Open Plan and usage in the dashboard and manage your plan in the Stripe customer portal. A move to Free takes effect at the end of the paid period, and your jobs keep running.",
  },
  {
    id: savingId,
    question: savingQuestion,
    answer: <SavingCalculation />,
  },
];

export function PricingFaq() {
  return (
    <Section id="questions">
      <div className="faq">
        <SectionHead
          title="Questions"
          lead={
            <>
              Plans, limits, and billing in short. The{" "}
              <TextLink to={paths.docsHome}>docs</TextLink> have the rest.
            </>
          }
        />
        <dl className="faq-list">
          {faqs.map((faq) => (
            <div key={faq.question} id={faq.id} className="faq-item">
              <dt className="faq-question">{faq.question}</dt>
              <dd className="faq-answer">{faq.answer}</dd>
            </div>
          ))}
        </dl>
      </div>
    </Section>
  );
}
