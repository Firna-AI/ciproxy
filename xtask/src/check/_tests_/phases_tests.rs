//! Phase order and include/exclude semantics.

use super::{MACOS_TARGET, Phase, commands, selected_phases};

#[test]
fn selected_phases_keep_fast_first_order_and_exclusion_wins() {
    assert_eq!(
        selected_phases(
            &[Phase::AppExport, Phase::RustFmt, Phase::RustTest],
            &[Phase::RustTest]
        ),
        vec![Phase::RustFmt, Phase::AppExport]
    );
    assert_eq!(
        selected_phases(&[], &[]),
        vec![
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
    );
}

#[test]
fn macos_phase_checks_then_lints_each_macos_crate_with_stub_c_tools() {
    assert_eq!(MACOS_TARGET, "aarch64-apple-darwin");
    let commands = commands(Phase::RustMacos);
    let args: Vec<String> = commands
        .iter()
        .map(|command| command.args.join(" "))
        .collect();
    assert_eq!(
        args,
        [
            "check --target aarch64-apple-darwin --package ciproxy-vz --all-targets",
            "clippy --target aarch64-apple-darwin --package ciproxy-vz --all-targets \
             --all-features -- -D warnings",
        ]
    );
    for command in commands {
        assert_eq!(command.program, "cargo");
        assert_eq!(
            command.root_paths,
            [
                ("CC_aarch64_apple_darwin", "xtask/macos-stubs/cc"),
                ("CXX_aarch64_apple_darwin", "xtask/macos-stubs/cc"),
                ("AR_aarch64_apple_darwin", "xtask/macos-stubs/ar"),
            ]
            .map(|(name, path)| (name.to_owned(), path.to_owned()))
        );
    }
}
