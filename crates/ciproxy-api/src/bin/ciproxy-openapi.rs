//! Emit the complete dashboard OpenAPI contract for the developer generator.

use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    serde_json::to_writer_pretty(io::stdout().lock(), &ciproxy_api::openapi::document())?;
    Ok(())
}
