//! Pinned RUST_LOG selection through the shared hosted subscriber.

use std::env::VarError;
use std::ffi::OsString;

use ciproxy_gateway::logging;
use tracing::subscriber::with_default;
use tracing_subscriber::fmt::writer::BoxMakeWriter;

#[path = "trace_logging/capture.rs"]
mod capture;

use capture::Capture;

#[test]
fn preserves_targets_defaults_fallbacks_and_configuration_notices() {
    let cases = [
        (Err(VarError::NotPresent), (true, true, false, false), None),
        (Ok("off".into()), (false, false, false, false), None),
        (Ok("".into()), (true, false, false, false), None),
        (
            Ok("probe=trace=off".into()),
            (false, false, false, false),
            Some(
                "Ignoring `RUST_LOG=\"probe=trace=off\"`: invalid filter directive: too many '=' in filter directive, expected 0 or 1",
            ),
        ),
        (
            Err(VarError::NotUnicode(OsString::from("invalid-unicode"))),
            (true, true, false, false),
            Some(
                "Ignoring `RUST_LOG`: environment variable was not valid unicode: \"invalid-unicode\"",
            ),
        ),
        (
            Ok("ciproxy_selection=debug".into()),
            (true, true, true, false),
            None,
        ),
        (Ok("trace".into()), (true, true, true, true), None),
    ];
    for (setting, expected, notice) in cases {
        let capture = Capture::default();
        let configured = logging::subscriber(setting.clone(), BoxMakeWriter::new(capture.clone()));
        assert_eq!(
            configured
                .notice
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            notice,
            "{setting:?}"
        );
        with_default(configured.subscriber, || {
            tracing::error!(target: "ciproxy_selection", "probe error");
            tracing::info!(target: "ciproxy_selection", "probe info");
            tracing::debug!(target: "ciproxy_selection", "probe debug");
            tracing::trace!(target: "ciproxy_selection", "probe trace");
        });
        let output = capture.text();
        assert_eq!(
            (
                output.contains("probe error"),
                output.contains("probe info"),
                output.contains("probe debug"),
                output.contains("probe trace")
            ),
            expected,
            "{setting:?}"
        );
    }
}
