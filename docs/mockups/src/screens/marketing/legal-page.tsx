// A legal page of ciproxy.dev, the terms of service or the privacy policy:
// the marketing frame, the title, the switch between the two pages, "On this
// page" on desktop, and the section headings. The user writes or approves the
// text of each section before the launch, so grey lines hold its place. Only
// the data retention of the privacy policy has text: the limits that
// security.md decides.

import { MockLink, type Viewport } from "@mokly/mokly";

import { paths, type CataloguePath } from "../../components/paths.ts";
import {
  historyWindows,
  legalPageOrder,
  legalPages,
  retentionId,
  retentionRows,
  type LegalPageKey,
  type LegalSection,
} from "../../fixtures/legal.ts";
import { MarketingShell } from "./marketing-shell.tsx";
import { Section, SectionHead } from "./section.tsx";
import "./legal.css";

const pagePaths: Readonly<Record<LegalPageKey, CataloguePath>> = {
  terms: paths.terms,
  privacy: paths.privacy,
};

/** The widths of the grey lines of a section, which vary by section. */
const lineSets: readonly (readonly number[])[] = [
  [100, 96, 88, 54],
  [100, 91, 72],
  [98, 100, 94, 83, 41],
];

function PageSwitch({ page }: { page: LegalPageKey }) {
  return (
    <nav className="legal-switch" aria-label="Legal pages">
      {legalPageOrder.map((key) => (
        <MockLink
          key={key}
          to={pagePaths[key]}
          className={`legal-switch-link${key === page ? " is-active" : ""}`}
          aria-current={key === page ? "page" : undefined}
        >
          {legalPages[key].title}
        </MockLink>
      ))}
    </nav>
  );
}

function Toc({
  page,
  sections,
}: {
  page: LegalPageKey;
  sections: readonly LegalSection[];
}) {
  return (
    <nav className="legal-toc" aria-label="On this page">
      <p className="legal-toc-title">On this page</p>
      <ul className="legal-toc-list">
        {sections.map((section) => (
          <li key={section.id}>
            <MockLink
              to={pagePaths[page]}
              fragment={section.id}
              className="legal-toc-link"
            >
              {section.title}
            </MockLink>
          </li>
        ))}
      </ul>
    </nav>
  );
}

/** The place of text that the user writes before the launch. */
function TextToCome({ index }: { index: number }) {
  const widths = lineSets[index % lineSets.length] ?? [];
  return (
    <div className="legal-lines" aria-hidden="true">
      {widths.map((width, line) => (
        <span
          key={line}
          className="legal-line"
          style={{ width: `${width}%` }}
        />
      ))}
    </div>
  );
}

function Retention() {
  return (
    <>
      <p>
        ciproxy keeps each kind of data for the period below, then deletes it.
      </p>
      <div className="legal-table-frame">
        <table className="legal-table">
          <thead>
            <tr>
              <th scope="col">Data</th>
              <th scope="col">Kept for</th>
            </tr>
          </thead>
          <tbody>
            {retentionRows.map((row) => (
              <tr key={row.data}>
                <td>{row.data}</td>
                <td>{row.kept}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p>Each plan shows only its own job history: {historyWindows()}.</p>
    </>
  );
}

export function LegalPageScreen({
  viewport,
  page,
}: {
  viewport: Viewport;
  page: LegalPageKey;
}) {
  const { title, sections } = legalPages[page];
  return (
    <MarketingShell viewport={viewport} section="legal">
      <Section className="legal-top">
        <SectionHead page eyebrow="Legal" title={title} />
        <PageSwitch page={page} />
      </Section>
      <Section className="legal-body">
        <div className="legal-grid">
          {viewport === "desktop" ? (
            <Toc page={page} sections={sections} />
          ) : null}
          <article className="legal-article">
            {sections.map((section, index) => (
              <section key={section.id} className="legal-section">
                <h2 id={section.id} className="legal-h2">
                  {section.title}
                </h2>
                {page === "privacy" && section.id === retentionId ? (
                  <Retention />
                ) : (
                  <TextToCome index={index} />
                )}
              </section>
            ))}
          </article>
        </div>
      </Section>
    </MarketingShell>
  );
}
