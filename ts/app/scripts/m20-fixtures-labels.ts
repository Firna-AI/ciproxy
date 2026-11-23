import type { components } from "../../api/index.ts";
import { fixtureTime } from "./fixtures-core.ts";
import { machineIds } from "./m20-fixtures-machines.ts";

type Schemas = components["schemas"];
type Fit = Schemas["LabelFit"];
type Label = Schemas["LabelView"];

export const machinePlace = { kind: "machine" } satisfies Schemas["LabelPlace"];
export const boatPlace = {
  kind: "overflow",
  provider: "boat",
} satisfies Schemas["LabelPlace"];

export const labelIds = {
  default: "60000000-0000-4000-8000-000000000001",
  medium: "60000000-0000-4000-8000-000000000002",
  large: "60000000-0000-4000-8000-000000000003",
  custom: "60000000-0000-4000-8000-000000000004",
} as const;

const gb = (value: number) => String(value * 1_000_000_000);

/** Two sizes of the Boat catalog of Northstar, with their whole disks. */
export const boatSizes = {
  large: {
    name: "large",
    size: { vcpus: 8, memory_bytes: gb(16), disk_bytes: gb(111) },
  },
  xlarge: {
    name: "xlarge",
    size: { vcpus: 16, memory_bytes: gb(32), disk_bytes: gb(251) },
  },
} as const satisfies Record<string, Schemas["ProviderCatalogSize"]>;

/**
 * The offers of the three eligible machines for a size with this VM disk:
 * forge-01 and office-mini-1, which run `jobs` of it, and forge-preparing,
 * whose capacity is not known yet.
 */
function machineFits(
  root: string,
  jobs: readonly [string, string],
): Schemas["LabelMachineFit"][] {
  const offer = { slots: "6", vcpus: "12", memory_bytes: gb(48) };
  return [
    {
      ...offer,
      id: machineIds.primary,
      hostname: "forge-01",
      job_disk_bytes: gb(645),
      required_root_disk_bytes: root,
      concurrent_jobs: jobs[0],
    },
    {
      ...offer,
      id: machineIds.diskLimited,
      hostname: "office-mini-1",
      job_disk_bytes: gb(279),
      required_root_disk_bytes: root,
      concurrent_jobs: jobs[1],
    },
    {
      ...offer,
      id: machineIds.unknownCapacity,
      hostname: "forge-preparing",
      job_disk_bytes: null,
      required_root_disk_bytes: null,
      concurrent_jobs: null,
    },
  ];
}

/** A size that both own machines and a Boat size fit. */
export function fixtureFit(
  root = gb(81),
  jobs: readonly [string, string] = ["6", "3"],
  boat: Schemas["ProviderCatalogSize"] = boatSizes.large,
): Fit {
  const counts = jobs.map(Number).filter((count) => count > 0);
  const places = counts.length === 0 ? [boatPlace] : [machinePlace, boatPlace];
  return {
    providers: [
      {
        provider: "boat",
        fit: {
          state: "fitting",
          refused: false,
          size: boat.size,
          size_name: boat.name,
        },
        connected: true,
        required_root_disk_bytes: root,
        largest_size: boatSizes.xlarge,
        hold: null,
      },
    ],
    eligible_machines: 3,
    fitting_machines: counts.length,
    machines: machineFits(root, jobs),
    minimum_jobs: counts.length === 0 ? null : String(Math.min(...counts)),
    maximum_jobs: counts.length === 0 ? null : String(Math.max(...counts)),
    no_machine_fits: counts.length === 0,
    fitting_places: places,
    routing: { kind: "places", places },
    unavailable_machines: 1,
  };
}

/** A size that no machine and no Boat size fits, so its jobs wait. */
export function nothingFits(root: string): Fit {
  return {
    ...fixtureFit(root, ["0", "0"]),
    providers: [
      {
        provider: "boat",
        connected: true,
        fit: { state: "no_size_fits" },
        largest_size: boatSizes.xlarge,
        required_root_disk_bytes: root,
      },
    ],
    eligible_machines: 2,
    fitting_machines: 0,
    machines: machineFits(root, ["0", "0"]).slice(0, 2),
    minimum_jobs: null,
    maximum_jobs: null,
    no_machine_fits: true,
    fitting_places: [],
    routing: { kind: "waits", reason: { kind: "nothing_fits" } },
    unavailable_machines: 0,
  };
}

/**
 * The fit of a label in a workspace without a Boat account, as after a setup
 * that skipped Boat: no catalog, so no Boat size, and the machines alone
 * decide where jobs run.
 */
export function withoutBoat(fit: Fit): Fit {
  const places = fit.fitting_machines > 0 ? [machinePlace] : [];
  return {
    ...fit,
    providers: fit.providers.map((provider) => ({
      ...provider,
      fit: { state: "unavailable" },
      connected: false,
      largest_size: null,
      hold: "not_connected",
    })),
    fitting_places: places,
    routing:
      places.length > 0
        ? { kind: "places", places }
        : {
            kind: "waits",
            reason: {
              kind: "provider",
              provider: "boat",
              hold: "not_connected",
            },
          },
  };
}

/**
 * The fit of a label with the switches of `input`, as the API places it: a
 * label without "Your machines" fits no machine, one without "Boat" has no
 * Boat fit, and the placement follows from what still fits.
 */
export function switchedFit(fit: Fit, input: Schemas["LabelInput"]): Fit {
  const machines =
    input.order.some((place) => place.kind === "machine") &&
    fit.fitting_machines > 0;
  const allowsBoat = input.order.some((place) => place.kind === "overflow");
  const providers = fit.providers.map(
    (provider): Schemas["ProviderLabelFit"] =>
      allowsBoat ? provider : { ...provider, fit: { state: "disabled" } },
  );
  const boatFits = providers.some(
    ({ fit }) => fit.state === "fitting" || fit.state === "largest_fallback",
  );
  const places = input.order.filter((place) =>
    place.kind === "machine" ? machines : boatFits,
  );
  return {
    ...fit,
    providers,
    fitting_machines: machines ? fit.fitting_machines : 0,
    minimum_jobs: machines ? (fit.minimum_jobs ?? null) : null,
    maximum_jobs: machines ? (fit.maximum_jobs ?? null) : null,
    no_machine_fits: !machines,
    fitting_places: places,
    routing:
      places.length > 0
        ? { kind: "places", places }
        : {
            kind: "waits",
            reason: {
              kind: input.order.length === 0 ? "empty_order" : "nothing_fits",
            },
          },
  };
}

function label(
  id: string,
  input: Partial<Schemas["LabelInput"]>,
  fit: Fit,
  jobs: string | null,
): Label {
  return {
    id,
    label: {
      name: "ciproxy",
      vcpus: 2,
      memory_gb: 8,
      disk_gb: 20,
      order: [machinePlace, boatPlace],
      when_nothing_fits: "wait",
      ...input,
    },
    fit,
    jobs_last_30_days: jobs,
    job_count_since: "2026-10-07T12:00:00Z",
    job_count_until: fixtureTime,
  };
}

export function fixtureLabels(): Label[] {
  return [
    label(labelIds.default, {}, fixtureFit(), "14371"),
    label(
      labelIds.medium,
      { name: "ciproxy-medium", vcpus: 4, memory_gb: 16 },
      fixtureFit(gb(81), ["3", "3"]),
      "502",
    ),
    label(
      labelIds.large,
      { name: "ciproxy-large", vcpus: 8, memory_gb: 32, disk_gb: 60 },
      fixtureFit(gb(122), ["1", "1"], boatSizes.xlarge),
      "2899",
    ),
    label(
      labelIds.custom,
      { name: "integration-tests", disk_gb: 1 },
      fixtureFit(gb(61), ["6", "4"]),
      null,
    ),
  ];
}
