// Two moments of the live queue in which a waiting label waits for the hold
// of the place that would run it, as the routing of the API gives it. In
// `held-off`, an Admin left own machines out of the order of ciproxy-16vcpu,
// so Boat is its one place, and the overflow switch is off. In
// `largest-held`, ciproxy-32vcpu runs on the largest size when nothing fits,
// and Boat, which would run it, reached its spend limit. Own machines have
// free slots in both, so no notice of a full queue shows.
import type { components } from "../../api/index.ts";
import { fixtureVmSize } from "./fixtures-jobs.ts";
import { boatPlace } from "./m20-fixtures-labels.ts";

type Schemas = components["schemas"];

export const heldModes = ["held-off", "largest-held"] as const;
export type HeldMode = (typeof heldModes)[number];

const xlarge = {
  vcpus: 16,
  memory_bytes: "64000000000",
  disk_bytes: "240000000000",
} satisfies Schemas["JobVmSize"];

/** The waiting label of a held moment, with the hold of Boat for it. */
function heldLabel(
  base: Schemas["WaitingLabelFit"],
  mode: HeldMode,
): Schemas["WaitingLabelFit"] {
  const off = mode === "held-off";
  const name = off ? "ciproxy-16vcpu" : "ciproxy-32vcpu";
  const hold: Schemas["ProviderHold"] = off ? "off" : "spend_limit";
  const provider: Schemas["ProviderLabelFit"] = {
    provider: "boat",
    fit: off
      ? {
          state: "fitting",
          size: fixtureVmSize,
          size_name: "4vcpu-8gb",
          refused: false,
        }
      : {
          state: "largest_fallback",
          size: xlarge,
          size_name: "xlarge",
          refused: false,
        },
    hold,
  };
  return {
    ...base,
    label_name: name,
    nothing_fits: false,
    providers: [provider],
    current: {
      id: base.label_id,
      label: {
        name,
        vcpus: off ? 16 : 32,
        memory_gb: off ? 64 : 128,
        disk_gb: off ? 60 : 200,
        order: off ? [boatPlace] : [{ kind: "machine" }, boatPlace],
        when_nothing_fits: off ? "wait" : "largest_overflow",
      },
      fit: {
        eligible_machines: 1,
        fitting_machines: 0,
        no_machine_fits: true,
        unavailable_machines: 0,
        minimum_jobs: null,
        maximum_jobs: null,
        providers: [provider],
        machines: [],
        fitting_places: [boatPlace],
        routing: {
          kind: "waits",
          reason: { kind: "provider", provider: "boat", hold },
        },
      },
    },
  };
}

/** The live queue of a held moment, from the live queue of the fixture. */
export function heldQueue(
  mode: HeldMode,
  page: Schemas["JobsLivePage"],
): Schemas["JobsLivePage"] {
  const off = mode === "held-off";
  const name = off ? "ciproxy-16vcpu" : "ciproxy-32vcpu";
  return {
    ...page,
    overflow_enabled: !off,
    starting_runners: [],
    waiting: page.waiting.map((job) => ({
      ...job,
      label: { ...job.label, name },
    })),
    waiting_labels: page.waiting_labels.map((label) => heldLabel(label, mode)),
    overflow: page.overflow.map((item) =>
      off ? item : { ...item, counted_cents: "10000", limit_reached: true },
    ),
  };
}
