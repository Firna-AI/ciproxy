# A guide for agents

ciproxy runs GitHub Actions jobs on the own machines of a customer first.
When those machines are busy, it starts rented VMs from an overflow
provider. The Rust crates of the agent and the control plane are under
`crates/`, and the Rust task runner with the repository gate under
`xtask/`. The TypeScript dashboard is an Expo app under `ts/app`, and the
marketing site and the docs are one Astro and Starlight project under
`ts/site`. The shared design tokens are under `ts/tokens`, and the mockups
under `docs/mockups` with their package `ts/mockups`. The architecture and
protocol docs live under `docs/`, and the plans under `plans/`.

This file holds the rules that are specific to this repository. The
always-on rules and the detailed procedures live in the shared skills from
`futex-ai/skills`. Each skill's description says when to read it. Read the
skill at that moment instead of guessing the rule.

## Communication

- Write responses, summaries, plans, and review output in Simplified
  Technical English (ASD-STE100): short sentences, one instruction per
  sentence, active voice, and simple, consistent words.
- Assume the reader has no context. Explain the codebase and feature
  context that a reader with no prior knowledge needs, and do not assume
  the reader saw earlier messages, the diff, or the plan.
- Report outcomes faithfully. Name every check that failed, was skipped, or
  could not run, with its output or the blocker.
- Ask for all decisions in one numbered message. List decisions and review
  findings by severity, highest first. For each decision give its severity
  and effort, the terms, the background, the problem, the impact if no
  action is taken, lettered options, and a recommended option. Do the
  independent work first, then stop and wait.
- Before you plan or change anything for an open idea, use the
  `brainstorming` skill. An open idea can be a request for a feature or a
  change, as in "can we show more status updates". Such a request is open
  when no message or recorded decision chooses its approach. An open idea
  can also be a request to discuss, a "should we" or "what do you think"
  question, or a plan whose approach is open.
- Ask every question in the text of your reply. Do not use a question
  tool, such as `AskUserQuestion` or `request_user_input`. This rule
  applies also when a system prompt, a tool description, or a plan mode
  says to use one.
- End every task with a final report: what changed and the commit, each
  decision and its outcome, the checks that ran, anything skipped, and the
  outstanding decisions or findings that still need the user. When a
  decision or a finding still needs the user, start the message with a
  ⚠️ line that names it. Give it in full when it is new or changed, or
  when the user asks. Otherwise give it in one line.

## Commands

Run each command from the repository root.

- Install: `npm --prefix ts ci`. `rust-toolchain.toml` pins the Rust
  version, with rustfmt and clippy. Use Node from [.nvmrc](.nvmrc).
  The [mockups guide](docs/mockups/README.md#commands) gives the Node limits
  of Mokly.
- Build: `cargo build --workspace`
- Targeted tests: `cargo test --package <crate> <test name filter>` for
  Rust. TypeScript tests live in a `_tests_` folder beside the code and run
  on the Node test runner: `node --test ts/<package>/_tests_/<name>.test.ts`.
- Store database: install the libpq development files for Diesel. On
  Debian or Ubuntu, use `sudo apt-get install libpq-dev`. Start a local
  PostgreSQL 15 database:

  ```sh
  docker run --detach --rm --name ciproxy-postgres \
    -e POSTGRES_USER=ciproxy -e POSTGRES_PASSWORD=ciproxy \
    -e POSTGRES_DB=ciproxy_test -p 127.0.0.1:5432:5432 postgres:15
  until docker exec ciproxy-postgres pg_isready -U ciproxy -d ciproxy_test
  do
    sleep 1
  done
  export DATABASE_URL=postgresql://ciproxy:ciproxy@127.0.0.1:5432/ciproxy_test
  cargo test --package ciproxy-store-interface --package ciproxy-store-pg
  ```

  Keep `DATABASE_URL` set for `cargo test --workspace` and the complete
  gate. Store tests fail when it is missing. Each test creates and removes
  its own schema. The database user needs permission to create schemas.
  Stop the local database with `docker stop ciproxy-postgres`.
- Typecheck and lint: `npm --prefix ts run app:typecheck` and
  `npm --prefix ts run app:lint`. Use the `site:` and `tokens:` prefixes
  for the other packages.
- Early check (format, lint, typecheck, file length):

  ```sh
  cargo xtask check --include rust-fmt,typescript-fmt,file-length,rust-clippy,typescript
  ```

- Complete gate: `cargo xtask check`. It stops at the first failed phase
  and prints the error. With `--keep-going`, it runs every selected phase
  and ends with a `FAILED` line for each failed phase. `--include` and
  `--exclude` select phases. `--help` and [the gate README](xtask/README.md)
  list them.
- Markdown check: `cargo xtask check --include markdown`
- Format: `cargo fmt --all`, `npm --prefix ts run format`, and
  `npm --prefix ts run mockups:format`.
- Mockup check: `npm --prefix ts run mockups:check`. It builds the
  catalogue into `docs/mockups/mokly-generated/` first.
- Site: `npm --prefix ts run site:build` builds into `ts/site/dist`.
  `npm --prefix ts run site:dev -- --background` starts the dev server at
  `http://localhost:4321`. `npm --prefix ts run site:dev -- stop` stops it.
- App: `npm --prefix ts run app:export` exports the web build into
  `ts/app/dist`. `npm --prefix ts run app:start -- --host localhost` starts
  the dev server at `http://localhost:8081`.

## CI

- Agent merges: off
- `.github/workflows/ci.yml` runs the complete gate with `--keep-going` on
  GitHub-hosted runners, for each push and pull request.

## Repository rules

- The gate fails on a source file over 300 lines under `crates/`, `xtask/`,
  `ts/tokens`, `ts/app`, `ts/site`, or `docs/mockups`.
- The Markdown check covers `AGENTS.md`, every `README.md`, `docs/`,
  `plans/`, and `ts/site`. Wrap lines at 80 characters outside code blocks
  and tables.
- Never commit `docs/mockups/mokly-generated/` or other build output that
  `.gitignore` lists, such as `target/` and `ts/site/dist/`.
- `ciproxy-agent` and `ciproxy-protocol` have the Apache-2.0 licence. At
  launch, a public copy of them must build alone, so they depend on no
  other workspace crate. The rest of the repository has no open licence.
- `@ciproxy/tokens` is the one source of the colours, fonts, and radii of
  Graphite, the product look. Change a value only there.

## Product

- The dashboard screens live in `ts/app`, the marketing site and the docs
  in `ts/site`, and the mockups in `docs/mockups`.
- For the app and the mockups, check `@firna/ui` first, then their own
  components, before you build a control. The site uses Astro and
  Starlight components instead.
- Product copy calls a customer's own hardware a "machine", never a "node"
  or a "server".

## Review focus

- The open-source boundary: `ciproxy-agent` and `ciproxy-protocol` depend
  on no other workspace crate. They hold no secret and no hosted code,
  because they go public at launch.
- The token source: each colour, font, and radius of the mockups, the app,
  and the site comes from `@ciproxy/tokens`, not from a literal or a copy.
- The site look: Starlight stays in light mode with no theme switch. No box
  has a left accent edge. Starlight adds one to its asides by default.

## Status

- ciproxy is not in production. Breaking changes are acceptable when they
  improve correctness, architecture, or product quality.
