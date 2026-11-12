//! The document, processed in Rust with the `compmd` crate: its expanded tree, its host components and its
//! diagnostics, in the JSON `@compmd/sandbox` reads.

use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use compmd::context::RenderContext;
use compmd::expand::Options;
use compmd::host::{Packs, Release};
use compmd::pack::{PackName, Place};
use compmd::{Ambient, ParseOptions, Processor};
use semver::Version;
use serde_json::{Value, json};

/// The document the app shows.
const DOCUMENT: &str = include_str!("../document.comp.md");

/// The Mermaid pack's name, and the version of it in this repository.
const MERMAID: &str = "@comp-markdown/mermaid";

/// The packs the document may import: the Mermaid pack, from this repository's `packs/mermaid/`, as an app with the
/// pack from npm would read it from `node_modules`.
struct Mermaid;

impl Mermaid {
    fn folder() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../packs/mermaid")
    }

    fn version() -> Option<Version> {
        let manifest: Value =
            serde_json::from_str(&fs::read_to_string(Self::folder().join("package.json")).ok()?)
                .ok()?;
        manifest["version"].as_str()?.parse().ok()
    }
}

impl Packs for Mermaid {
    fn releases(&self, pack: &PackName) -> Vec<Release> {
        let release = Self::version().map(|version| Release {
            version,
            integrity: None,
            broken: None,
        });
        if *pack == MERMAID {
            release.into_iter().collect()
        } else {
            Vec::new()
        }
    }

    fn pack_file(&self, place: &Place) -> Option<String> {
        let ours = place.release.pack == MERMAID
            && Self::version().as_ref() == Some(&place.release.version);
        let file = place
            .path
            .parts()
            .fold(Self::folder(), |file, part| file.join(part));
        ours.then(|| fs::read_to_string(file).ok()).flatten()
    }
}

/// The document processed at `now` (milliseconds since the epoch), in UTC.
#[must_use]
pub fn processed(now: i128) -> Value {
    let context = RenderContext {
        now,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    };
    let processor = Processor::new("document.comp.md", DOCUMENT)
        .parse(&ParseOptions::default())
        .resolve((), Mermaid, &Ambient::default())
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
    use super::*;

    #[test]
    fn the_document_processes_with_its_diagram_and_its_component() {
        let processed = processed(0);
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
        assert_eq!(
            processed["hostComponents"][1]["trust"],
            json!({ "level": "strict" })
        );
    }
}
