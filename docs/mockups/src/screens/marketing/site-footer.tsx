// The footer of the marketing pages: the wordmark with one line about the
// product, three columns of links, and the copyright with the GitHub mark of
// the open-source agent.

import { Github } from "lucide-react";
import { MockLink } from "@mokly/mokly";

import { Brand } from "../../components/brand.tsx";
import { paths, type CataloguePath } from "../../components/paths.ts";
import { copyrightYear } from "../../fixtures/site.ts";
import "./site-footer.css";

type FooterLink = { label: string; to?: CataloguePath };

const columns: readonly { title: string; links: readonly FooterLink[] }[] = [
  {
    title: "Product",
    links: [
      { label: "Pricing", to: paths.pricing },
      { label: "Docs", to: paths.docsHome },
      { label: "Sign in", to: paths.signIn },
    ],
  },
  {
    title: "Docs",
    links: [
      { label: "Quickstart", to: paths.quickstart },
      { label: "Labels and machine sizes", to: paths.labelsReference },
      { label: "Security" },
    ],
  },
  {
    title: "Legal",
    links: [{ label: "Terms" }, { label: "Privacy policy" }],
  },
];

function FooterItem({ link }: { link: FooterLink }) {
  return link.to ? (
    <MockLink to={link.to} className="foot-link">
      {link.label}
    </MockLink>
  ) : (
    <span className="foot-link">{link.label}</span>
  );
}

export function SiteFooter() {
  return (
    <footer className="foot">
      <div className="foot-top">
        <div className="foot-brand">
          <Brand />
          <p className="small muted">GitHub Actions on your own machines.</p>
        </div>
        <nav className="foot-columns" aria-label="Footer">
          {columns.map((column) => (
            <div key={column.title} className="foot-column">
              <p className="foot-title">{column.title}</p>
              <ul className="foot-list">
                {column.links.map((link) => (
                  <li key={link.label}>
                    <FooterItem link={link} />
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </nav>
      </div>
      <div className="foot-base">
        <span className="caption">
          © {copyrightYear} ciproxy. The agent is open source under Apache-2.0.
        </span>
        <span className="foot-icon" title="The ciproxy agent on GitHub">
          <Github size={16} strokeWidth={1.75} aria-label="GitHub" />
        </span>
      </div>
    </footer>
  );
}
