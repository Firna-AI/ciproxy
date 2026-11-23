//! Convert macOS VM counters into job usage figures.

#[cfg(target_os = "macos")]
mod mach;

#[cfg(target_os = "macos")]
pub(crate) use self::mach::Mach;

fn memory_mib(
    internal: u32,
    purgeable: u32,
    wired: u32,
    compressed: u32,
    page_size: usize,
) -> Option<u32> {
    let app = u128::from(internal.saturating_sub(purgeable));
    let pages = app + u128::from(wired) + u128::from(compressed);
    let bytes = pages * page_size as u128;
    u32::try_from(bytes / (1_024 * 1_024)).ok()
}

#[cfg(test)]
#[path = "_tests_/macos_tests.rs"]
mod macos_tests;
