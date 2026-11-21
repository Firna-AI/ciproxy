// The frame of a marketing page: the site top bar, then the page sections and
// the footer between two hairline rails, in the manner of the Vercel site.

import type { ReactNode } from "react";
import type { Viewport } from "@mokly/mokly";

import {
  SiteHeader,
  type SiteSection,
} from "../../components/site/site-header.tsx";
import { SiteFooter } from "./site-footer.tsx";
import "./marketing.css";

export function MarketingShell({
  viewport,
  section,
  children,
}: {
  viewport: Viewport;
  section: SiteSection;
  children: ReactNode;
}) {
  return (
    <div className={`site-frame mk mk-${viewport}`}>
      <SiteHeader viewport={viewport} section={section} />
      <div className="mk-rails">
        <main>{children}</main>
        <SiteFooter />
      </div>
    </div>
  );
}
