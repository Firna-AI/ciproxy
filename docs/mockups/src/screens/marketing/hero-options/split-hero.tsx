// A hero option with the text on the left and a picture on the right, on the
// white page between the rails. A phone and a narrow window stack the two.
// The accent option draws the picture and the second title line in the
// proposed accent of @ciproxy/tokens.

import type { ReactNode } from "react";

import { HeroText } from "../hero.tsx";
import "./hero-options.css";

export function SplitHero({
  picture,
  accent = false,
}: {
  picture: ReactNode;
  accent?: boolean;
}) {
  return (
    <section className={`hero ho ho-split${accent ? " is-accent" : ""}`}>
      <div className="hero-copy ho-copy">
        <HeroText center={false} />
      </div>
      <div className="ho-picture">{picture}</div>
    </section>
  );
}
