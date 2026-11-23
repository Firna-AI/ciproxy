// The last sections of the docs page How ciproxy works (how-ciproxy-works.tsx):
// slots and disk, overflow to Boat, the cache, and the job image. A link to a
// docs page that has no mockup yet, such as Connect Boat, shows as its plain
// title.

import { TextLink } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";
import { slotSize } from "../../fixtures/hardware.ts";
import { boat } from "../../fixtures/machines.ts";
import { sizePhrase } from "../../fixtures/sizes.ts";
import { features, includes, tierOrder, tiers } from "../../fixtures/tiers.ts";
import { DocsH2 } from "./docs-parts.tsx";

/** The plans with sticky Docker disks, from the tier record. */
function stickyDiskPlans(): string {
  const names = tierOrder
    .filter((tier) => includes(tier, features.stickyDisks))
    .map((tier) => tiers[tier].name);
  const last = names.pop() ?? "";
  return names.length > 0 ? `${names.join(", ")} and ${last}` : last;
}

export function SlotsSection() {
  return (
    <>
      <DocsH2 id="slots-and-disk">Slots and disk</DocsH2>
      <p>
        A slot is {sizePhrase(slotSize)} of memory of a machine. The machine
        keeps a quarter of its threads and memory for the host, and the rest
        forms slots. A job takes as many slots as its size needs.
      </p>
      <p>
        A running job also holds the disk of its VM: the image and the free disk
        of its label. A machine first keeps disk for the images, then a quarter
        of the rest for the host. The machine cache uses the disk that the jobs
        leave.{" "}
        <TextLink to={paths.labelsReference}>Labels and VM sizes</TextLink>{" "}
        gives the numbers.
      </p>
    </>
  );
}

export function OverflowSection() {
  return (
    <>
      <DocsH2 id="overflow-to-boat">Overflow to Boat</DocsH2>
      <ul className="docs-list">
        <li>
          A job that one of your machines can run waits there for room, up to
          the overflow wait. The wait is {boat.waitSeconds} seconds by default.
          Then ciproxy starts one Boat runner for the job.
        </li>
        <li>
          When none of your machines can run the job, for example because its
          size fits none of them, ciproxy starts the Boat runner at once.
        </li>
        <li>
          Each job on Boat runs in its own sandbox. ciproxy chooses the cheapest
          Boat size that covers its label. When none fits, the label&apos;s{" "}
          <TextLink to={paths.labelsReference} fragment="custom-labels">
            When nothing fits
          </TextLink>{" "}
          setting can allow the largest Boat size instead. ciproxy deletes the
          sandbox when the job ends.
        </li>
        <li>
          ciproxy starts a Boat runner only when you connected a Boat account,
          overflow is on, the label allows Boat, and the spend limit has room.
          Otherwise the job waits for your machines.
        </li>
        <li>ciproxy never runs a job again by itself.</li>
      </ul>
      <p>Connect Boat and Set a spend limit give the settings.</p>
    </>
  );
}

export function CacheSection() {
  return (
    <>
      <DocsH2 id="the-cache">The cache</DocsH2>
      <ul className="docs-list">
        <li>
          <span className="inline-code">actions/cache</span> and other actions
          that use the GitHub cache work without workflow edits.
        </li>
        <li>
          On your machines, a job restores an entry from the machine cache when
          the machine has it, and from the GitHub cache otherwise. A save goes
          to both.
        </li>
        <li>A job on Boat uses the GitHub cache only.</li>
        <li>
          Only jobs on the default branch write entries that every branch can
          restore. Job isolation gives the rules.
        </li>
        <li>
          A failure of the machine cache never fails a job. The job then uses
          the GitHub cache.
        </li>
        <li>
          <strong>Clear cache</strong> on the page of a machine deletes every
          entry of its machine cache. It never deletes an entry of the GitHub
          cache.
        </li>
      </ul>
      <p>
        On {stickyDiskPlans()}, sticky Docker disks keep the Docker data of each
        repository on each machine between its jobs. Docker disks gives the
        rules.
      </p>
    </>
  );
}

export function JobImageSection() {
  return (
    <>
      <DocsH2 id="the-job-image">The job image</DocsH2>
      <p>
        Every job starts from one image with the software of the GitHub runner
        image of <span className="inline-code">ubuntu-latest</span>, on your
        machines and on Boat. So a workflow keeps its steps, and only its{" "}
        <span className="inline-code">runs-on</span> line changes.
      </p>
      <p>
        A job can run its steps in any Docker image, with the{" "}
        <span className="inline-code">container:</span> key of GitHub Actions,
        on every plan. Run steps in another image gives the full example.
      </p>
    </>
  );
}
