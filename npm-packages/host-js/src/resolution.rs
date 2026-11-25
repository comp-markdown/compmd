//! The registry rules' resolution (`compmd_host::registry`) as WebAssembly, for `@compmd/registry` ([the registry
//! rules], decisions 1 and 2): its steps and answers as JSON, a tarball's bytes as bytes. The rules do no I/O, so the
//! binding awaits nothing: `@compmd/registry` drives the steps with `fetch` and Cache Storage, and gives each answer.
//!
//! [the registry rules]: https://github.com/comp-markdown/compmd/blob/main/docs/registry-rules.md

use std::time::Duration;

use compmd_host::pack::PackName;
use compmd_host::registry::{Answer, Next, Options, Resolution};
use serde_json::json;
use wasm_bindgen::prelude::{JsError, wasm_bindgen};

/// A resolution of a request's packs.
#[wasm_bindgen(js_name = Resolution)]
pub struct JsResolution(Resolution);

#[wasm_bindgen(js_class = Resolution)]
impl JsResolution {
    /// A resolution with the host's `options`, as JSON: `{"registries", "maxPackSize", "deny"}`.
    ///
    /// # Errors
    ///
    /// When `options` aren't a resolution's.
    #[wasm_bindgen(constructor)]
    pub fn new(options: &str) -> Result<Self, JsError> {
        let options: Options = serde_json::from_str(options)?;
        Ok(Self(Resolution::new(options)))
    }

    /// What the host is to do next, as JSON: `{"steps": [...]}`, each step with its `kind` and `id`, or
    /// `{"done": resolved}`.
    #[wasm_bindgen(js_name = nextSteps)]
    #[must_use]
    pub fn next_steps(&mut self) -> String {
        let next = match self.0.next_steps() {
            Next::Steps(steps) => json!({ "steps": steps }),
            Next::Done(resolved) => json!({ "done": resolved }),
        };
        next.to_string()
    }

    /// The host's answer to the step `id`, as JSON, with its `kind`.
    ///
    /// # Errors
    ///
    /// When `answer` isn't an answer.
    pub fn give(&mut self, id: u32, answer: &str) -> Result<(), JsError> {
        let answer: Answer = serde_json::from_str(answer)?;
        self.0.give(u64::from(id), answer);
        Ok(())
    }

    /// A tarball's `bytes`, the answer to the step `id`: as bytes, not JSON.
    #[wasm_bindgen(js_name = giveTarball)]
    pub fn give_tarball(&mut self, id: u32, bytes: &[u8]) {
        self.0.give(
            u64::from(id),
            Answer::Tarball {
                bytes: bytes.to_vec(),
            },
        );
    }

    /// Gives up on `pack`, which hasn't come within `milliseconds`.
    ///
    /// # Errors
    ///
    /// When `pack` isn't a pack's name.
    #[wasm_bindgen(js_name = giveUp)]
    pub fn give_up(&mut self, pack: &str, milliseconds: f64) -> Result<(), JsError> {
        let pack: PackName = pack.parse()?;
        self.0.give_up(
            &pack,
            Duration::from_secs_f64(milliseconds.max(0.0) / 1000.0),
        );
        Ok(())
    }
}

/// The versions of `pack` an npm registry at `registry` answered with `status` and `body`, as JSON:
/// `{"versions": [...]}`, or `{"error": why}`.
#[wasm_bindgen(js_name = answeredVersions)]
#[must_use]
pub fn answered_versions(registry: &str, pack: &str, status: u16, body: &str) -> String {
    let answered = pack
        .parse::<PackName>()
        .map_err(|error| error.to_string())
        .and_then(|pack| compmd_host::registry::answered_versions(registry, &pack, status, body));
    match answered {
        Ok(versions) => json!({ "versions": versions }),
        Err(error) => json!({ "error": error }),
    }
    .to_string()
}

/// Why the tarball at `tarball` may not be fetched for the npm registry at `registry`, if so: it isn't on the
/// registry's origin.
#[wasm_bindgen(js_name = offOrigin)]
#[must_use]
pub fn off_origin(tarball: &str, registry: &str) -> Option<String> {
    compmd_host::registry::off_origin(tarball, registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn a_resolution_with_nothing_to_import_is_done_after_its_first_step() {
        let mut resolution =
            JsResolution::new(r#"{"registries": {"*": "https://registry.npmjs.org/"}}"#)
                .unwrap_or_else(|_| panic!("options"));
        let next: Value = serde_json::from_str(&resolution.next_steps()).unwrap_or(Value::Null);
        assert_eq!(next["steps"][0]["kind"], "missing");
        let id =
            u32::try_from(next["steps"][0]["id"].as_u64().unwrap_or_default()).unwrap_or_default();
        assert!(
            resolution
                .give(id, r#"{"kind": "missing", "packs": []}"#)
                .is_ok()
        );
        let done: Value = serde_json::from_str(&resolution.next_steps()).unwrap_or(Value::Null);
        assert_eq!(done["done"]["problems"], json!([]));
        resolution.give_tarball(99, b"ignored");
        assert!(resolution.give_up("@kit/x", 50.0).is_ok());
    }

    #[test]
    fn npm_answers_are_read_as_the_crate_reads_them() {
        let none: Value =
            serde_json::from_str(&answered_versions("https://r.example/", "@kit/x", 404, ""))
                .unwrap_or(Value::Null);
        assert_eq!(none, json!({ "versions": [] }));
        let failed: Value =
            serde_json::from_str(&answered_versions("https://r.example/", "@kit/x", 500, ""))
                .unwrap_or(Value::Null);
        assert_eq!(
            failed["error"],
            "https://r.example/ answered 500 for `@kit/x`"
        );
        let wrong: Value =
            serde_json::from_str(&answered_versions("https://r.example/", "x", 200, "{}"))
                .unwrap_or(Value::Null);
        assert!(wrong["error"].is_string());
        assert!(off_origin("https://elsewhere.example/t.tgz", "https://r.example/").is_some());
    }
}
