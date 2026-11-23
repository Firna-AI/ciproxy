// The workflow change that moves a job to ciproxy, as the code blocks show
// it. A line that starts with - or + is a diff line.

/** The workflow file of the examples. */
export const workflowFile = ".github/workflows/ci.yml";

/** The one line that changes, with the lines around it. */
export const runsOnDiff: readonly string[] = [
  "  jobs:",
  "    test:",
  "-     runs-on: ubuntu-latest",
  "+     runs-on: ciproxy",
];

/** The steps of the job, after the runs-on line. */
export const jobSteps: readonly string[] = [
  "      steps:",
  "        - uses: actions/checkout@v4",
  "        - run: cargo test --workspace",
];

/**
 * A whole workflow file of mokly-ai, as the texture of a hero variation:
 * three jobs, each on ciproxy.
 */
export const heroWorkflow: readonly string[] = [
  "name: ci",
  "on:",
  "  push:",
  "    branches: [main]",
  "  pull_request:",
  "",
  "jobs:",
  "  lint:",
  "    runs-on: ciproxy",
  "    steps:",
  "      - uses: actions/checkout@v4",
  "      - run: npm ci",
  "      - run: npm run lint",
  "",
  "  test:",
  "    runs-on: ciproxy",
  "    strategy:",
  "      matrix:",
  "        shard: [1, 2, 3, 4]",
  "    steps:",
  "      - uses: actions/checkout@v4",
  "      - run: cargo test --workspace",
  "",
  "  build:",
  "    needs: [lint, test]",
  "    runs-on: ciproxy",
  "    steps:",
  "      - uses: actions/checkout@v4",
  "      - run: cargo build --release",
];
