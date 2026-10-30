//! The components a document can name (spec §6.3), for a tool that lists them: grouped by the file that defines
//! them, each with its kind, its signature and where the document uses it.

use crate::diagnostic::Range;
use crate::resolve::{
    Body, Component, DocumentId, Origin, Renderer, Resolved, code_blocks, invocations,
};
use crate::tree::Signature;
use serde::Serialize;
use std::collections::BTreeMap;

/// The components a processed document can name, and the names it uses that stand for none.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Components {
    /// The components by the file that defines them: the document first, then the files it imports in written
    /// order (those an import leads to further, in the order they were loaded), then the packs the processor
    /// provides.
    pub groups: Vec<Group>,
    /// The names the document uses that stand for no one component: unknown (F002), ambiguous (F023) or from an
    /// import that failed.
    pub unknown: Vec<Used>,
}

/// The components one file defines.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Group {
    /// The file, as diagnostics name it; `None` for the processed document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Whether the processor provides it: `@compmd/core` or an ambient pack.
    pub provided: bool,
    /// Its components, by the name the document gives them.
    pub components: Vec<Listed>,
}

/// A component the document can name.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Listed {
    /// The name the document gives it.
    pub name: String,
    /// Its name in the file that defines it, when an import renames it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_as: Option<String>,
    /// What implements it: `compmd`, `processor`, or the host language as written (`jsx`, `svelte`, …).
    pub kind: String,
    /// Its signature; `None` for a host-language component without one, whose params go unchecked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
    /// Where the processed document defines it: its definition fence; `None` for one another file defines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined: Option<Range>,
    /// Where the document invokes it, each the range of the name.
    pub uses: Vec<Range>,
}

/// A name the document uses, and where.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Used {
    /// The name.
    pub name: String,
    /// Where the document invokes it, each the range of the name.
    pub uses: Vec<Range>,
}

/// The components the processed document of `resolved` can name.
pub(crate) fn of(resolved: &Resolved) -> Components {
    let document = resolved.document(resolved.main);
    let lines = document.lines();
    let mut uses: BTreeMap<&str, Vec<Range>> = BTreeMap::new();
    let mut found = Vec::new();
    invocations(&document.root.children, &mut found);
    for invocation in found {
        let at = lines.name_at(invocation.start);
        uses.entry(invocation.name)
            .or_default()
            .push(Range::spanning(at, invocation.name));
    }
    // A code block a code renderer renders uses it, at the block's language (spec §7.7).
    let mut blocks = Vec::new();
    code_blocks(&document.root.children, &mut blocks);
    for code in blocks {
        let (Some(language), Some(position)) = (code.lang.as_deref(), code.position) else {
            continue;
        };
        if let Some(Renderer::Component(name, _)) = document.renderers.get(language) {
            let at = lines.language_at(position.start);
            uses.entry(name)
                .or_default()
                .push(Range::spanning(at, language));
        }
    }
    for list in uses.values_mut() {
        list.sort_by_key(|range| (range.start.line, range.start.column));
    }
    // Each group with the file it is and the document that loaded it, which orders it.
    let mut groups: Vec<(Origin, DocumentId, Group)> = Vec::new();
    for (name, id) in document.names.components() {
        let component = resolved.component(id);
        let defining = resolved.document(component.document);
        let origin = origin(resolved, component);
        let listed = Listed {
            name: name.to_string(),
            defined_as: (component.name != name).then(|| component.name.clone()),
            kind: kind(component),
            signature: component.signature.clone(),
            defined: component
                .position
                .filter(|_| origin == document.origin)
                .map(Range::from),
            uses: uses.remove(name).unwrap_or_default(),
        };
        if let Some((.., group)) = groups.iter_mut().find(|(known, ..)| *known == origin) {
            group.components.push(listed);
        } else {
            let group = Group {
                file: (origin != document.origin).then(|| origin.to_string()),
                provided: defining.provided,
                components: vec![listed],
            };
            groups.push((origin, component.document, group));
        }
    }
    let imports = &document.settings.imports;
    let written = |origin: &Origin| {
        imports
            .iter()
            .position(|import| document.origin.join(&import.from.value).as_ref() == Some(origin))
            .unwrap_or(imports.len())
    };
    groups.sort_by_key(|(origin, loaded, group)| {
        (
            group.file.is_some(),
            group.provided,
            written(origin),
            *loaded,
        )
    });
    // What is left stands for no component.
    let unknown = uses
        .into_iter()
        .map(|(name, uses)| Used {
            name: name.to_string(),
            uses,
        })
        .collect();
    Components {
        groups: groups.into_iter().map(|(.., group)| group).collect(),
        unknown,
    }
}

/// What implements `component`, as [`Listed::kind`] says it.
pub(crate) fn kind(component: &Component) -> String {
    match &component.body {
        Body::Compmd(_) => "compmd".to_string(),
        Body::Processor => "processor".to_string(),
        Body::Host { language, .. } => language.clone(),
        Body::File { language, .. } => language.name().to_string(),
    }
}

/// The file that defines `component`: the file a document imports it from, or the document whose fence defines it.
pub(crate) fn origin(resolved: &Resolved, component: &Component) -> Origin {
    match &component.body {
        Body::File { origin, .. } => origin.clone(),
        _ => resolved.document(component.document).origin.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::context::RenderContext;
    use crate::expand::Options;
    use crate::{Ambient, ParseOptions, Processor};
    use serde_json::{Value, json};
    use std::collections::HashMap;
    use std::rc::Rc;

    /// The components of `text` with `files`, as JSON.
    fn components(text: &str, files: &[(&str, &str)]) -> Value {
        let files: HashMap<String, String> = files
            .iter()
            .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
            .collect();
        let context = RenderContext {
            now: 0,
            tz: "UTC".into(),
            time: Rc::new(()),
            inputs: serde_json::Map::default(),
        };
        let processor = Processor::new("a.comp.md", text)
            .parse(&ParseOptions::default())
            .resolve(files, (), &Ambient::default())
            .validate(&[])
            .load(context, (), ())
            .expand(&Options::default());
        serde_json::to_value(processor.components()).expect("JSON")
    }

    /// The range of a name on `line` from `column`, `length` code points long.
    fn at(line: usize, column: usize, length: usize) -> Value {
        json!({ "start": { "line": line, "column": column }, "end": { "line": line, "column": column + length } })
    }

    #[test]
    fn groups_the_documents_own_then_its_imports_then_core() {
        let text = "---\nmd:\n  import:\n    - from: ./kit.comp.md\n      names: [{name: badge, as: tag}]\n    - from: ./Chart.jsx\n      names: [{name: Chart, as: chart}]\n---\n\n```compmd component=note\n---\n:: tag[]\n```\n\n:: note[]\n\n::: box\n:chart[] and :note[]\n::: /box\n";
        let kit = "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=badge\nparams:\n  tone: string?\n---\nB.\n```\n";
        let json = components(
            text,
            &[
                ("kit.comp.md", kit),
                ("Chart.jsx", "export default () => null;"),
            ],
        );
        let groups = json["groups"].as_array().expect("groups");
        assert_eq!(groups[0]["file"], Value::Null);
        assert_eq!(
            groups[0]["components"],
            json!([{
                "name": "note",
                "kind": "compmd",
                "signature": { "params": [], "slots": null, "headings": "relative" },
                "defined": { "start": { "line": 10, "column": 1 }, "end": { "line": 13, "column": 4 } },
                "uses": [at(15, 4, 4), at(18, 15, 4)]
            }])
        );
        assert_eq!(groups[1]["file"], "kit.comp.md");
        let tag = &groups[1]["components"][0];
        assert_eq!(tag.get("defined"), None, "defined in another file");
        assert_eq!(
            (
                &tag["name"],
                &tag["definedAs"],
                &tag["signature"]["params"][0]["name"]
            ),
            (&json!("tag"), &json!("badge"), &json!("tone"))
        );
        assert_eq!(
            tag["uses"],
            json!([at(12, 4, 3)]),
            "inside a definition too"
        );
        assert_eq!(groups[2]["file"], "Chart.jsx");
        assert_eq!(groups[2]["components"][0]["kind"], "jsx");
        assert_eq!(groups[2]["components"][0].get("signature"), None);
        let core = &groups[3];
        assert_eq!(
            (&core["file"], &core["provided"]),
            (&json!("@compmd/core@1.0.0/index.comp.md"), &json!(true))
        );
        let used: Vec<&Value> = core["components"]
            .as_array()
            .expect("core")
            .iter()
            .filter(|component| component["uses"] != json!([]))
            .collect();
        assert_eq!(used.len(), 1);
        assert_eq!(
            (&used[0]["name"], &used[0]["kind"]),
            (&json!("box"), &json!("processor"))
        );
        assert_eq!(groups.len(), 4);
        assert_eq!(json["unknown"], json!([]));
    }

    #[test]
    fn a_code_renderer_is_used_by_its_blocks_at_their_language() {
        let text = "```compmd component=shout\nrenders: [shout, yell]\n---\n{{code}}\n```\n\n```yell\nhi\n```\n\n:: shout[Hi] {code=\"hi\"}\n\n```sh\nno\n```\n";
        let json = components(text, &[]);
        let shout = &json["groups"][0]["components"][0];
        assert_eq!(shout["signature"]["renders"], json!(["shout", "yell"]));
        assert_eq!(shout["signature"]["params"][0]["name"], "code");
        assert_eq!(shout["uses"], json!([at(7, 4, 4), at(11, 4, 5)]));
    }

    #[test]
    fn names_that_stand_for_no_component_are_unknown() {
        let json = components(":: nothing[]\n\n:: nothing[]\n", &[]);
        assert_eq!(
            json["unknown"],
            json!([{ "name": "nothing", "uses": [at(1, 4, 7), at(3, 4, 7)] }])
        );
    }
}
