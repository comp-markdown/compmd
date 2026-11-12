//! The document, processed in Rust with the `compmd` crate over the packs the app serves (`app.compmd().packs()`: the
//! bundle's, then the store's, the same files the frames load): its expanded tree, its host components and its
//! diagnostics, in the JSON `@compmd/sandbox` reads.

use std::rc::Rc;

use compmd::context::RenderContext;
use compmd::expand::Options;
use compmd::host::Packs;
use compmd::{Ambient, ParseOptions, Processor};
use serde_json::{Value, json};

/// The document the app shows.
const DOCUMENT: &str = include_str!("../document.comp.md");

/// The document processed at `now` (milliseconds since the epoch), in UTC, with `packs`.
#[must_use]
pub fn processed(now: i128, packs: impl Packs + 'static) -> Value {
    let context = RenderContext {
        now,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    };
    let processor = Processor::new("document.comp.md", DOCUMENT)
        .parse(&ParseOptions::default())
        .resolve((), packs, &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&Options::default());
    json!({
        "tree": processor.expanded(),
        "hostComponents": processor.host_components(),
        "diagnostics": processor.diagnostics(),
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use tauri_plugin_compmd::compmd_host::{Bundle, HostPacks, NoStore};

    /// The bundle the page's build wrote, as the app embeds it.
    fn bundle() -> Bundle {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("compmd");
        Bundle::read(&folder).expect("the bundle: build the page first (`pnpm build`)")
    }

    #[test]
    fn the_document_processes_with_its_diagram_and_its_component() {
        let processed = processed(0, HostPacks::new(Arc::new(bundle()), Arc::new(NoStore)));
        assert_eq!(
            processed["diagnostics"],
            json!([]),
            "{}",
            processed["diagnostics"]
        );
        let names: Vec<&str> = processed["hostComponents"]
            .as_array()
            .expect("a list")
            .iter()
            .filter_map(|component| component["name"].as_str())
            .collect();
        assert_eq!(names, ["mermaid", "probe"]);
        // The Mermaid pack's built module, by its place in the bundle, which the frames load by URL.
        assert_eq!(
            processed["hostComponents"][0]["module"]["pack"],
            json!("@comp-markdown/mermaid")
        );
        assert_eq!(
            processed["hostComponents"][1]["trust"],
            json!({ "level": "strict" })
        );
    }

    #[test]
    fn without_packs_the_import_is_refused() {
        let processed = processed(0, ());
        let codes: Vec<&str> = processed["diagnostics"]
            .as_array()
            .expect("a list")
            .iter()
            .filter_map(|diagnostic| diagnostic["code"].as_str())
            .collect();
        assert!(codes.contains(&"F026"), "{codes:?}");
    }
}
