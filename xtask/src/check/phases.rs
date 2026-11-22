//! Pure phase selection and command definitions, in execution order.

use clap::ValueEnum;

use crate::command::CommandSpec;

/// The target of the macOS agent and the macOS VM layer.
pub(crate) const MACOS_TARGET: &str = "aarch64-apple-darwin";

/// The crates with macOS code and the cargo target options to check. The
/// `rust-macos` phase checks each of them for [`MACOS_TARGET`]. Use
/// `["--lib", "--bins"]` for a crate whose tests do not build for macOS yet.
pub(crate) const MACOS_CRATES: &[(&str, &[&str])] = &[
    ("ciproxy-vz", &["--all-targets"]),
    ("ciproxy-image-vm", &["--all-targets"]),
    // The agent test targets use Linux fixtures, so the check covers the
    // library and the binary only.
    ("ciproxy-agent", &["--lib", "--bins"]),
];

/// Stub C tools, so build scripts with C code pass a type check on a host
/// with no Apple SDK. The check never links.
const MACOS_STUB_TOOLS: [(&str, &str); 3] = [
    ("CC_aarch64_apple_darwin", "xtask/macos-stubs/cc"),
    ("CXX_aarch64_apple_darwin", "xtask/macos-stubs/cc"),
    ("AR_aarch64_apple_darwin", "xtask/macos-stubs/ar"),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum Phase {
    /// Rust format: cargo fmt --all -- --check.
    RustFmt,
    /// Prettier format checks for tokens, app, and site.
    TypescriptFmt,
    /// Source files in crates, xtask, tokens, app, and site: 300 lines.
    FileLength,
    /// Maintained Markdown: local links and markdownlint-cli2.
    Markdown,
    /// Rust lint: cargo clippy --workspace --all-targets -- -D warnings.
    RustClippy,
    /// cargo check and cargo clippy of the macOS crates for aarch64-apple-darwin.
    RustMacos,
    /// ESLint and typecheck of tokens, app, and site, plus token unit tests.
    Typescript,
    /// Regenerate Rust-owned OpenAPI and TypeScript, then fail on drift.
    ApiTypes,
    /// Rust tests and doctests: cargo test --workspace.
    RustTest,
    /// Existing mockup build, typecheck, lint, format, and policy check.
    Mockups,
    /// Astro and Starlight production build.
    SiteBuild,
    /// Expo web export.
    AppExport,
}

pub(crate) fn selected_phases(include: &[Phase], exclude: &[Phase]) -> Vec<Phase> {
    [
        Phase::RustFmt,
        Phase::TypescriptFmt,
        Phase::FileLength,
        Phase::Markdown,
        Phase::RustClippy,
        Phase::RustMacos,
        Phase::Typescript,
        Phase::ApiTypes,
        Phase::RustTest,
        Phase::Mockups,
        Phase::SiteBuild,
        Phase::AppExport,
    ]
    .into_iter()
    .filter(|phase| (include.is_empty() || include.contains(phase)) && !exclude.contains(phase))
    .collect()
}

pub(super) fn commands(phase: Phase) -> Vec<CommandSpec> {
    match phase {
        Phase::RustFmt => vec![CommandSpec::new(
            "cargo",
            &["fmt", "--all", "--", "--check"],
        )],
        Phase::TypescriptFmt => vec![npm_script("format:check")],
        Phase::FileLength | Phase::Markdown => Vec::new(),
        Phase::RustClippy => vec![CommandSpec::new(
            "cargo",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        )],
        Phase::RustMacos => macos_commands(),
        Phase::Typescript => vec![
            npm_script("tokens:check"),
            npm_script("app:lint"),
            npm_script("app:typecheck"),
            npm_script("site:lint"),
            npm_script("site:typecheck"),
            npm_script("api:lint"),
            npm_script("api:typecheck"),
        ],
        Phase::ApiTypes => vec![npm_script("api:generate:check")],
        Phase::RustTest => vec![CommandSpec::new("cargo", &["test", "--workspace"])],
        Phase::Mockups => vec![npm_script("mockups:check")],
        Phase::SiteBuild => vec![npm_script("site:build")],
        Phase::AppExport => vec![npm_script("app:export")],
    }
}

/// Check the default features, then lint every feature, of each macOS crate.
fn macos_commands() -> Vec<CommandSpec> {
    MACOS_CRATES
        .iter()
        .flat_map(|(package, targets)| {
            let common = ["--target", MACOS_TARGET, "--package", package];
            let check = [&["check"], &common[..], targets].concat();
            let clippy = [
                &["clippy"],
                &common[..],
                targets,
                &["--all-features", "--", "-D", "warnings"],
            ]
            .concat();
            [macos_command(&check), macos_command(&clippy)]
        })
        .collect()
}

fn macos_command(args: &[&str]) -> CommandSpec {
    MACOS_STUB_TOOLS
        .iter()
        .fold(CommandSpec::new("cargo", args), |command, (name, path)| {
            command.root_path(name, path)
        })
}

fn npm_script(script: &str) -> CommandSpec {
    CommandSpec::new("npm", &["--prefix", "ts", "run", script])
}

#[cfg(test)]
#[path = "_tests_/phases_tests.rs"]
mod phases_tests;
