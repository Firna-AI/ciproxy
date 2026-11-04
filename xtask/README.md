# xtask

The repository gate for ciproxy.
This private developer tool runs the same checks locally and in CI.
It has no open licence and contains no product behaviour.

## Responsibilities

Run the approved checks in a fixed order.
Stop at the first failed phase by default.
Run all selected phases when `--keep-going` is set.
Keep check logic independent of disk and process access.

## What This Crate Does

The gate has these phases, in execution order:

| Phase | Check or command |
| --- | --- |
| `rust-fmt` | `cargo fmt --all -- --check` |
| `typescript-fmt` | `npm --prefix ts run format:check` |
| `file-length` | The 300-line source limit |
| `markdown` | Local Markdown links, then the pinned `markdownlint-cli2` |
| `rust-clippy` | `cargo clippy --workspace --all-targets -- -D warnings` |
| `typescript` | `tokens:check`, `app:lint`, `app:typecheck`, `site:lint`, `site:typecheck` |
| `rust-test` | `cargo test --workspace`, including doctests |
| `mockups` | `npm --prefix ts run mockups:check` |
| `site-build` | `npm --prefix ts run site:build` |
| `app-export` | `npm --prefix ts run app:export` |

Every TypeScript script in the table runs with `npm --prefix ts run`.
`tokens:check` runs ESLint, strict typecheck, and the CSS builder unit test.
The site typecheck uses `astro check`.
Format uses Prettier, with its Astro plugin for the site.

The length check reads Rust under `crates/` and `xtask/`.
It also reads TypeScript, JavaScript, Astro, and CSS under `ts/tokens/`,
`ts/app/`, and `ts/site/`.
It skips dependencies, lockfiles, and generated output.
The mockups retain their own length check.

Markdown checks cover `AGENTS.md`, all `README.md` files, `docs/`, `plans/`,
and the Markdown content of `ts/site/`.
They skip `CLAUDE.md`, dependencies, build output, and `.context/`.
Local links resolve from the source file and use GitHub heading IDs.
A `?plain=1` link with `#L5`, `#L5-L9`, or `#L5C2-L9C4` is a line link.
Line links need no heading.
The `#top` fragment needs no heading, also without `?plain=1`.
It matches without regard to ASCII case.
URLs with a scheme and site-root routes are outside the local link check.
The lint uses default rules with MD013 disabled for code blocks and tables.

## Quick Start

`rust-toolchain.toml` pins Rust, with rustfmt and clippy.
`.cargo/config.toml` holds the `cargo xtask` alias.
Use the Node version in [.nvmrc](../.nvmrc).
Use the npm version in the `packageManager` field of
[ts/package.json](../ts/package.json).
Run these commands from the repository root:

```sh
npm --prefix ts ci
cargo build --workspace
cargo xtask check
cargo xtask check --help
```

Use `--keep-going` to run all selected phases after a failure.
The gate then prints one summary line for each failed phase at the end.
It exits with a failure if any selected phase failed.
CI uses this mode so every selected phase has a result:

```sh
cargo xtask check --keep-going
```

Select phases with `--include` or `--exclude`.
Repeat an option or separate phase names with commas.
The execution order stays fixed, and exclusion takes priority.

```sh
cargo xtask check --include rust-fmt,typescript-fmt,file-length
cargo xtask check --include markdown
cargo xtask check --exclude markdown
```

The Markdown phase invokes this command with the discovered files appended:

```sh
npm --prefix ts exec -- markdownlint-cli2 --config xtask/markdownlint-cli2.jsonc
```

## Development

```sh
cargo test --package xtask
cargo test --package xtask missing_relative_file_reports_its_source_line
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
npm --prefix ts run format
npm --prefix ts run mockups:check
npm --prefix ts run site:build
npm --prefix ts run app:export
npm --prefix ts run site:dev -- --host 0.0.0.0
npm --prefix ts run app:start -- --host localhost
```

### Key Code

- [src/cli/args.rs](src/cli/args.rs) defines commands and phase filters.
- [src/check/phases.rs](src/check/phases.rs) orders phases and their commands.
- [src/check/runner.rs](src/check/runner.rs) runs phases and reports failures.
- [src/files.rs](src/files.rs) discovers files through a repository boundary.
- [src/file_length.rs](src/file_length.rs) checks the source length limit.
- [src/markdown/links.rs](src/markdown/links.rs) validates local link targets.
- [src/command.rs](src/command.rs) defines the subprocess boundary.
- [markdownlint-cli2.jsonc](markdownlint-cli2.jsonc) configures Markdown lint.

### Related Docs

- [Tokens](../ts/tokens/README.md) owns the shared Graphite values.
- [Dashboard](../ts/app/README.md) documents the web app commands.
- [Site](../ts/site/README.md) documents the marketing and docs build.
