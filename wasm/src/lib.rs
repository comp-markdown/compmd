//! The CompMD processor as WebAssembly. A request and its result cross as JSON text (the `compmd` crate's
//! [`Request`] and [`Processed`](compmd::process::Processed)); time zone offsets come from the page's `Intl`, so the
//! module carries no zone data. The npm package `@compmd/processor` wraps it with types.

use compmd::md::TimeZones;
use compmd::process::Request;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/zones.js")]
extern "C" {
    /// The offset of an IANA zone from UTC at a time in seconds since 1970, in seconds east of UTC; `undefined` for
    /// a zone `Intl` doesn't know.
    #[wasm_bindgen(js_name = zoneOffset)]
    fn zone_offset(zone: &str, seconds: f64) -> Option<f64>;
}

/// The page's time zones, through `Intl`.
struct Intl;

impl TimeZones for Intl {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        reason = "times in CEL's range and offsets in seconds are exact as doubles"
    )]
    fn offset(&self, zone: &str, at: i64) -> Option<i32> {
        zone_offset(zone, at as f64).map(|seconds| seconds as i32)
    }
}

/// Runs stages 1 to 4b on `request`: `{parse, expanded, diagnostics}`.
///
/// # Errors
///
/// A request that isn't one, or whose document isn't one of its files.
#[wasm_bindgen]
pub fn process(request: &str) -> Result<String, JsError> {
    let request: Request = serde_json::from_str(request)?;
    let processed = compmd::process::process(&request, &Intl)?;
    Ok(serde_json::to_string(&processed)?)
}

/// The remote and `host:` sources a render of `request` needs: `{remote, host}`.
///
/// # Errors
///
/// A request that isn't one, or whose document isn't one of its files.
#[wasm_bindgen]
pub fn sources(request: &str) -> Result<String, JsError> {
    let request: Request = serde_json::from_str(request)?;
    let needed = compmd::process::sources(&request, &Intl)?;
    Ok(serde_json::to_string(&needed)?)
}
