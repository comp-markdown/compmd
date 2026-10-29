//! A document kept open for an editor: processed once, then changed by edits in the editor's UTF-16 positions. After
//! each change it gives, as JSON (`camelCase`) that `src/index.ts` types, the splice of the expanded tree, every
//! diagnostic, and the host-language components when they changed: three pieces, so a host can pass the splice on
//! as text without reading it.

use crate::request::{self, Error};
use crate::text::Text;
use compmd::host::{Fetched, Time};
use compmd::processor::{Expanded, HostComponent};
use compmd::{Processor, TextEdit, context::RenderContext, expanded};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use std::collections::HashMap;
use std::rc::Rc;

/// The processor of an open document, its text, the render context its loads use, its host-language components,
/// and the last edit.
pub(crate) struct Document {
    text: Text,
    processor: Processor<Expanded>,
    context: RenderContext,
    components: Vec<HostComponent>,
    /// Whether the last change changed the components, so that it gives them.
    components_changed: bool,
    /// The last change's edit, in code points; none after opening or a reload.
    edit: Option<TextEdit>,
}

/// A host-language component, for the renderer to run (`compmd::processor::HostComponent`).
#[derive(Serialize)]
struct Component<'a> {
    name: &'a str,
    language: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime: Option<&'a str>,
    code: &'a str,
}

impl<'a> From<&'a HostComponent> for Component<'a> {
    fn from(component: &'a HostComponent) -> Self {
        Self {
            name: &component.name,
            language: component.language.name(),
            runtime: component.runtime.as_deref(),
            code: &component.code,
        }
    }
}

/// The processed document's top-level expanded nodes `from..to` became `nodes`; every node after them moved by
/// `lines` lines and `offset` code points. With the root's inputs, and the edit that made it.
#[derive(Serialize)]
struct Splice<'a> {
    from: usize,
    to: usize,
    nodes: &'a [expanded::Node],
    lines: isize,
    offset: isize,
    #[serde(skip_serializing_if = "expanded::Inputs::is_empty")]
    inputs: &'a expanded::Inputs,
    #[serde(skip_serializing_if = "Option::is_none")]
    edit: Option<Edit<'a>>,
}

/// An edit in code points (spec §10.5): the text's `start..end` became `text`.
#[derive(Serialize)]
struct Edit<'a> {
    start: usize,
    end: usize,
    text: &'a str,
}

impl<'a> From<&'a TextEdit> for Edit<'a> {
    fn from(edit: &'a TextEdit) -> Self {
        Self {
            start: edit.start,
            end: edit.end,
            text: &edit.text,
        }
    }
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
        let processor = prepared.expand();
        Ok(Self {
            text,
            components: processor.host_components(),
            components_changed: true,
            processor,
            context,
            edit: None,
        })
    }

    /// The last change's splice, `{from, to, nodes, lines, offset, inputs, edit}`; after opening, the whole tree.
    pub(crate) fn splice(&self) -> Result<String, Error> {
        let splice = self.processor.splice();
        Ok(serde_json::to_string(&Splice {
            from: splice.from,
            to: splice.to,
            nodes: &splice.nodes,
            lines: splice.lines,
            offset: splice.offset,
            inputs: &self.processor.expanded().inputs,
            edit: self.edit.as_ref().map(Edit::from),
        })?)
    }

    /// Every diagnostic of the document, in stage order.
    pub(crate) fn diagnostics(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(self.processor.diagnostics())?)
    }

    /// The host-language components, when the last change changed them (and after opening).
    pub(crate) fn components(&self) -> Result<Option<String>, Error> {
        self.components_changed
            .then(|| {
                let components: Vec<_> = self.components.iter().map(Component::from).collect();
                serde_json::to_string(&components)
            })
            .transpose()
            .map_err(Error::from)
    }

    /// The document after its text's UTF-16 `start..end` became `text`.
    pub(crate) fn edit(mut self, start: usize, end: usize, text: String) -> Self {
        let edit = self.text.edit(start, end, text);
        let processor = self.processor.edit(&edit);
        Self {
            processor,
            edit: Some(edit),
            ..self
        }
        .settled()
    }

    /// With the components the processor gives now, noting whether they changed.
    fn settled(self) -> Self {
        let components = self.processor.host_components();
        Self {
            components_changed: components != self.components,
            components,
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
        Self {
            processor,
            edit: None,
            ..self
        }
        .settled()
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

    /// The last change's three pieces, `components` null when they didn't change.
    fn changes(document: &Document) -> Json {
        let parse = |json: &str| serde_json::from_str::<Json>(json).expect("JSON");
        json!({
            "splice": parse(&document.splice().expect("splice")),
            "diagnostics": parse(&document.diagnostics().expect("diagnostics")),
            "components": document.components().expect("components").as_deref().map(parse),
        })
    }

    #[test]
    fn opening_gives_the_whole_tree() {
        let document = open("One.\n\nTwo.\n");
        let changes = changes(&document);
        let splice = &changes["splice"];
        assert_eq!((&splice["from"], &splice["to"]), (&json!(0), &json!(0)));
        assert_eq!(splice["nodes"].as_array().map(Vec::len), Some(2));
        assert_eq!(changes["diagnostics"], json!([]));
        assert!(splice.get("inputs").is_none());
        assert!(splice.get("edit").is_none(), "opening isn't an edit");
        assert!(Document::open("{}", Rc::new(())).is_err());
        let with_inputs = open("---\nmd:\n  inputs:\n    n: number = 1\n---\n");
        assert_eq!(
            self::changes(&with_inputs)["splice"]["inputs"]["n"]["value"],
            1
        );
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
        assert_eq!(
            splice["edit"],
            json!({ "start": 5, "end": 5, "text": "!" }),
            "the edit, in code points"
        );
        assert_eq!(changes["diagnostics"][0]["code"], "F002");
    }

    #[test]
    fn components_come_on_open_and_when_their_code_changes() {
        let text = "```jsx component=hi runtime=react@19\nexport default () => 'hi';\n```\n\n:: hi[Hi]\n\nText.\n";
        let document = open(text);
        assert_eq!(
            changes(&document)["components"],
            json!([{ "name": "hi", "language": "jsx", "runtime": "react@19", "code": "export default () => 'hi';" }])
        );
        let end = text.len() - 1;
        let document = document.edit(end, end, "!".into());
        assert!(
            changes(&document)["components"].is_null(),
            "a paragraph changed, not the code"
        );
        let quote = text.find("'hi'").expect("the code") + 1;
        let document = document.edit(quote, quote + 2, "hello".into());
        assert_eq!(
            changes(&document)["components"][0]["code"],
            "export default () => 'hello';"
        );
        let plain = open("```js component=p\nexport default () => {};\n```\n");
        assert!(changes(&plain)["components"][0].get("runtime").is_none());
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
        assert!(
            changes["splice"].get("edit").is_none(),
            "a reload isn't an edit"
        );
        assert_eq!(changes["diagnostics"], json!([]));
        assert!(Data::from_json("{\"remote\": []}").is_err());
    }
}
