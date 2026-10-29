//! A document kept open for an editor: processed once, then changed by edits in the editor's UTF-16 positions, each
//! giving what changed in the expanded tree as JSON (`camelCase`), as `src/index.ts` types it.

use crate::request::{self, Error};
use crate::text::Text;
use compmd::diagnostic::Diagnostic;
use compmd::host::{Fetched, Time};
use compmd::processor::Expanded;
use compmd::{Processor, context::RenderContext, expanded};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use std::collections::HashMap;
use std::rc::Rc;

/// The processor of an open document, its text, and the render context its loads use.
pub(crate) struct Document {
    text: Text,
    processor: Processor<Expanded>,
    context: RenderContext,
}

/// What the last change gave: the splice of the expanded tree, the root's inputs, and every diagnostic.
#[derive(Serialize)]
struct Changes<'a> {
    splice: Splice<'a>,
    #[serde(skip_serializing_if = "expanded::Inputs::is_empty")]
    inputs: &'a expanded::Inputs,
    diagnostics: &'a [Diagnostic],
}

/// The processed document's top-level expanded nodes `from..to` became `nodes`; every node after them moved by
/// `lines` lines and `offset` code points.
#[derive(Serialize)]
struct Splice<'a> {
    from: usize,
    to: usize,
    nodes: &'a [expanded::Node],
    lines: isize,
    offset: isize,
}

/// Data the host fetched, for a reload: remote data by URL, and `host:` data by source.
#[derive(Default, Deserialize)]
#[serde(default)]
pub(crate) struct Data {
    remote: HashMap<String, Fetched>,
    host: HashMap<String, Json>,
}

impl Data {
    /// The data in `json`, `{remote, host}`.
    pub(crate) fn from_json(json: &str) -> Result<Self, Error> {
        Ok(serde_json::from_str(json)?)
    }
}

impl Document {
    /// The request in `json`, processed.
    pub(crate) fn open(json: &str, time: Rc<dyn Time>) -> Result<Self, Error> {
        let mut prepared = request::prepare(json, time)?;
        let text = Text::new(std::mem::take(&mut prepared.text));
        let context = prepared.context.clone();
        Ok(Self {
            text,
            processor: prepared.expand(),
            context,
        })
    }

    /// What the last change gave, `{splice, diagnostics}`; after opening, the splice is the whole tree.
    pub(crate) fn changes(&self) -> Result<String, Error> {
        let splice = self.processor.splice();
        let changes = Changes {
            splice: Splice {
                from: splice.from,
                to: splice.to,
                nodes: &splice.nodes,
                lines: splice.lines,
                offset: splice.offset,
            },
            inputs: &self.processor.expanded().inputs,
            diagnostics: self.processor.diagnostics(),
        };
        Ok(serde_json::to_string(&changes)?)
    }

    /// The document after its text's UTF-16 `start..end` became `text`.
    pub(crate) fn edit(mut self, start: usize, end: usize, text: String) -> Self {
        let edit = self.text.edit(start, end, text);
        Self {
            processor: self.processor.edit(&edit),
            ..self
        }
    }

    /// The remote and `host:` sources a render needs now, `{remote, host}`.
    pub(crate) fn sources(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(&self.processor.sources())?)
    }

    /// The document loaded and expanded again with `data`.
    pub(crate) fn reload(self, data: Data) -> Self {
        let processor = self
            .processor
            .reload(self.context.clone(), data.remote, data.host);
        Self { processor, ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn open(text: &str) -> Document {
        let request = json!({
            "path": "a.comp.md",
            "files": { "a.comp.md": text },
            "allowedOrigins": ["https://a.org"],
            "now": "2026-10-22T09:00:00Z",
            "origins": true
        });
        Document::open(&request.to_string(), Rc::new(())).expect("opened")
    }

    fn changes(document: &Document) -> Json {
        serde_json::from_str(&document.changes().expect("changes")).expect("JSON")
    }

    #[test]
    fn opening_gives_the_whole_tree() {
        let document = open("One.\n\nTwo.\n");
        let changes = changes(&document);
        assert_eq!(
            (&changes["splice"]["from"], &changes["splice"]["to"]),
            (&json!(0), &json!(0))
        );
        assert_eq!(changes["splice"]["nodes"].as_array().map(Vec::len), Some(2));
        assert_eq!(changes["diagnostics"], json!([]));
        assert!(changes.get("inputs").is_none());
        assert!(Document::open("{}", Rc::new(())).is_err());
        let with_inputs = open("---\nmd:\n  inputs:\n    n: number = 1\n---\n");
        assert_eq!(self::changes(&with_inputs)["inputs"]["n"]["value"], 1);
    }

    #[test]
    fn an_edit_in_utf16_positions_gives_its_splice() {
        // `😀` is two UTF-16 units and one code point: typing after `One` is at 6 for JavaScript, 5 for the processor.
        let document = open("😀 One.\n\nTwo.\n\n:: nope[N]\n").edit(6, 6, "!".into());
        let changes = changes(&document);
        let splice = &changes["splice"];
        assert_eq!(
            (
                &splice["from"],
                &splice["to"],
                &splice["lines"],
                &splice["offset"]
            ),
            (&json!(0), &json!(1), &json!(0), &json!(1))
        );
        assert_eq!(splice["nodes"][0]["children"][0]["value"], "😀 One!.");
        assert_eq!(
            splice["nodes"][0]["origin"]["end"],
            json!({ "line": 1, "column": 8, "offset": 7 })
        );
        assert_eq!(changes["diagnostics"][0]["code"], "F002");
    }

    #[test]
    fn sources_and_reload_follow_the_frontmatter() {
        let frontmatter = "---\nmd:\n  data:\n    - from: https://a.org/x.json\n      as: x\n---\n";
        let document = open("Text.\n").edit(0, 0, frontmatter.into()).edit(
            99,
            99,
            "\n{{md.data.x.n}}\n".into(),
        );
        let needed: Json =
            serde_json::from_str(&document.sources().expect("sources")).expect("JSON");
        assert_eq!(
            needed,
            json!({ "remote": ["https://a.org/x.json"], "host": [] })
        );
        assert_eq!(changes(&document)["diagnostics"][0]["code"], "F026");
        let fetched = json!({ "remote": { "https://a.org/x.json": { "text": "{\"n\": \"fetched\"}", "contentType": "application/json" } } });
        let data = Data::from_json(&fetched.to_string()).expect("data");
        let changes = changes(&document.reload(data));
        assert_eq!(
            (&changes["splice"]["from"], &changes["splice"]["to"]),
            (&json!(0), &json!(2))
        );
        assert_eq!(
            changes["splice"]["nodes"][1]["children"][0]["value"],
            "fetched"
        );
        assert_eq!(changes["diagnostics"], json!([]));
        assert!(Data::from_json("{\"remote\": []}").is_err());
    }
}
