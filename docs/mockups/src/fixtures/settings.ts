// Labels, shapes, and repositories of mokly-ai.

export type Label = {
  name: string;
  vcpu: number;
  memoryGb: number;
  diskGb: number;
  /** Jobs in the last 30 days. */
  jobs: number;
};

export const labels: readonly Label[] = [
  { name: "ciproxy", vcpu: 2, memoryGb: 8, diskGb: 60, jobs: 14_371 },
  { name: "ciproxy-4vcpu", vcpu: 4, memoryGb: 16, diskGb: 80, jobs: 502 },
  { name: "ciproxy-8vcpu", vcpu: 8, memoryGb: 32, diskGb: 120, jobs: 2_899 },
];

/** Overflow wait times the customer can pick, in seconds. */
export const waitChoices = [0, 15, 30, 60, 120] as const;

export type Repository = { name: string; selected: boolean };

/** The 14 private repositories that the GitHub App can see. */
export const repositories: readonly Repository[] = [
  { name: "mokly", selected: true },
  { name: "mokly-cloud", selected: true },
  { name: "mokly-site", selected: true },
  { name: "mokly-docs", selected: true },
  { name: "mokly-action", selected: true },
  { name: "viewer-kit", selected: true },
  { name: "design-tokens", selected: false },
  { name: "storybook-adapter", selected: true },
  { name: "examples", selected: false },
  { name: "infra", selected: true },
  { name: "release-tools", selected: true },
  { name: "benchmarks", selected: false },
  { name: "cloud-billing", selected: true },
  { name: "homebrew-tap", selected: false },
];
