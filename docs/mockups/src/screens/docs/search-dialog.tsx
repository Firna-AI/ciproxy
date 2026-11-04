// The search dialog of the docs, as Starlight opens it from the top bar: the
// query, the matching pages with their headings and excerpts, and the keys.
// A centred dialog on desktop, the whole screen on mobile. The match is in
// bold ink; colour marks status only.

import {
  ArrowDown,
  ArrowUp,
  CornerDownLeft,
  FileText,
  Search,
} from "lucide-react";
import { MockLink, type Viewport } from "@mokly/mokly";

import { Action } from "../../components/actions.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { topId } from "./docs-parts.tsx";
import "./search.css";

const query = "runs-on";

type Hit = { heading: string; fragment: string; before: string; after: string };

type PageHits = { page: string; to: CataloguePath; hits: readonly Hit[] };

const results: readonly PageHits[] = [
  {
    page: "Quickstart",
    to: paths.quickstart,
    hits: [
      {
        heading: "Change runs-on",
        fragment: "change-runs-on",
        before: "In a workflow file, change the ",
        after: " line of a job.",
      },
    ],
  },
  {
    page: "Labels and VM sizes",
    to: paths.labelsReference,
    hits: [
      {
        heading: "Labels and VM sizes",
        fragment: topId,
        before: "The label in ",
        after: " picks the size of the VM that runs the job.",
      },
      {
        heading: "Default labels",
        fragment: "default-labels",
        before: "",
        after: ": ciproxy-8vcpu",
      },
    ],
  },
  {
    page: "Overview",
    to: paths.docsHome,
    hits: [
      {
        heading: "How a job runs",
        fragment: "how-a-job-runs",
        before: "GitHub queues a job whose workflow says ",
        after: ": ciproxy.",
      },
    ],
  },
];

function HitLink({
  to,
  hit,
  selected,
}: {
  to: CataloguePath;
  hit: Hit;
  selected: boolean;
}) {
  return (
    <MockLink
      to={to}
      fragment={hit.fragment}
      className={`ds-hit${selected ? " is-selected" : ""}`}
    >
      <span className="ds-hit-heading">{hit.heading}</span>
      <span className="ds-hit-excerpt">
        {hit.before}
        <mark className="ds-mark">{query}</mark>
        {hit.after}
      </span>
      {selected ? <CornerDownLeft size={14} className="ds-enter" /> : null}
    </MockLink>
  );
}

export function SearchDialog({ viewport }: { viewport: Viewport }) {
  const count = results.reduce((sum, page) => sum + page.hits.length, 0);
  return (
    <div className={`ds-scrim ds-${viewport}`}>
      <section className="ds" role="dialog" aria-label="Search docs">
        <div className="ds-field">
          <Search size={16} className="icon-muted" />
          <input
            className="ds-input"
            value={query}
            readOnly
            aria-label="Search docs"
          />
          {viewport === "mobile" ? (
            <Action tone="ghost" size="sm">
              Cancel
            </Action>
          ) : (
            <kbd className="site-kbd">esc</kbd>
          )}
        </div>
        <p className="ds-count caption">
          {count} results for {query}
        </p>
        <ul className="ds-results">
          {results.map((page, pageIndex) => (
            <li key={page.page} className="ds-page">
              <span className="ds-page-title">
                <FileText size={14} className="icon-muted" />
                {page.page}
              </span>
              {page.hits.map((hit, hitIndex) => (
                <HitLink
                  key={hit.fragment}
                  to={page.to}
                  hit={hit}
                  selected={
                    viewport === "desktop" && pageIndex === 0 && hitIndex === 0
                  }
                />
              ))}
            </li>
          ))}
        </ul>
        {viewport === "desktop" ? (
          <p className="ds-keys caption">
            <kbd className="site-kbd" aria-label="Up">
              <ArrowUp size={11} />
            </kbd>
            <kbd className="site-kbd" aria-label="Down">
              <ArrowDown size={11} />
            </kbd>
            <span>to move,</span>
            <kbd className="site-kbd" aria-label="Enter">
              <CornerDownLeft size={11} />
            </kbd>
            <span>to open</span>
          </p>
        ) : null}
      </section>
    </div>
  );
}
