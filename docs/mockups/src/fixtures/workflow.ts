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
