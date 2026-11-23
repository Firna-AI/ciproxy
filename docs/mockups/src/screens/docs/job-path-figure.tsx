// The picture of the docs page How ciproxy works: the parts of ciproxy and
// the path of one job, from the queue in GitHub to the end of its VM, with
// the other way of a job to a sandbox in the Boat account. The numbers are
// the steps of "A job on your machines". Boxes in a CSS grid with an arrow
// between neighbours, from the tokens, so that the site builds the same
// picture with plain HTML and CSS. The text is in reading order: the path
// first, then the other way. The arrows and the icons are hidden from screen
// readers. The figure carries data-pagefind-ignore, so that search finds the
// text of the page and not this short copy of it.

import { Fragment, type ReactNode } from "react";
import { ArrowDown, Box, Cloud, Github, Server } from "lucide-react";

import { BrandMark } from "../../components/brand.tsx";
import "./job-path-figure.css";

/** The part that a box shows. It also names the place of the box. */
type PartKey = "github" | "ciproxy" | "machine" | "runner" | "boat";

type PathBox = {
  part: PartKey;
  /** The number of the step in "A job on your machines". Boat has none. */
  step?: number;
  title: string;
  text: string;
};

const iconSize = 16;

const icons: Readonly<Record<PartKey, ReactNode>> = {
  github: <Github size={iconSize} strokeWidth={1.75} />,
  ciproxy: <BrandMark size={iconSize} />,
  machine: <Server size={iconSize} strokeWidth={1.75} />,
  runner: <Box size={iconSize} strokeWidth={1.75} />,
  boat: <Cloud size={iconSize} strokeWidth={1.75} />,
};

/** The path of a job on your machines, in the order of its steps. */
const path: readonly PathBox[] = [
  {
    part: "github",
    step: 1,
    title: "GitHub",
    text: "Queues a job of a ciproxy label.",
  },
  {
    part: "ciproxy",
    step: 2,
    title: "ciproxy",
    text: "Decides how many runners start, and where.",
  },
  {
    part: "machine",
    step: 3,
    title: "Your machine",
    text: "The agent starts a fresh VM with a runner.",
  },
  {
    part: "runner",
    step: 4,
    title: "The runner",
    text: "Takes a job from GitHub and runs it. Then the VM ends.",
  },
];

/** The other way of a job: from ciproxy to your Boat account. */
const boat: PathBox = {
  part: "boat",
  title: "Your Boat account",
  text: "Runs the job in its own sandbox.",
};

/** When a job takes the other way. It stays true for a macOS job. */
const boatWhen =
  "Or, when your machines have no room and Boat can take the job";

const caption =
  "A job on ciproxy, from the queue in GitHub to the end of its VM.";

/** An arrow to a box. A wide figure turns the arrows of its first row. */
function Arrow({ to }: { to: PartKey }) {
  return (
    <span className={`job-path-arrow job-path-to-${to}`} aria-hidden="true">
      <ArrowDown size={iconSize} strokeWidth={1.5} />
    </span>
  );
}

function PathStep({ box }: { box: PathBox }) {
  return (
    <div className={`job-path-box job-path-${box.part}`}>
      {box.step === undefined ? null : (
        <span className="job-path-number">
          {String(box.step).padStart(2, "0")}
        </span>
      )}
      <span className="job-path-icon" aria-hidden="true">
        {icons[box.part]}
      </span>
      <p className="job-path-title">{box.title}</p>
      <p className="job-path-text">{box.text}</p>
    </div>
  );
}

export function JobPathFigure() {
  return (
    <figure className="job-path" data-pagefind-ignore="">
      <div className="job-path-grid">
        {path.map((box, index) => (
          <Fragment key={box.part}>
            {index > 0 ? <Arrow to={box.part} /> : null}
            <PathStep box={box} />
          </Fragment>
        ))}
        <div className="job-path-when">
          <p className="job-path-when-text">{boatWhen}</p>
          <Arrow to={boat.part} />
        </div>
        <PathStep box={boat} />
      </div>
      <figcaption className="job-path-caption">{caption}</figcaption>
    </figure>
  );
}
