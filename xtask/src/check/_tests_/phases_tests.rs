//! Phase order and include/exclude semantics.

use super::{Phase, selected_phases};

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
            Phase::Typescript,
            Phase::ApiTypes,
            Phase::RustTest,
            Phase::Mockups,
            Phase::SiteBuild,
            Phase::AppExport,
        ]
    );
}
