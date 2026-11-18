//! Examples (spec §7.8) beyond the conformance set: their host-language components apart from the document's, their
//! origins and diagnostics at their places in the document, and what they need from the host.

use compmd::context::RenderContext;
use compmd::expand::Options;
use compmd::{Ambient, Language, ParseOptions, Processor};
use std::collections::HashMap;
use std::rc::Rc;

fn context() -> RenderContext {
    RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    }
}

fn process(text: &str, options: &Options) -> Processor<compmd::processor::Expanded> {
    let files: HashMap<String, String> = HashMap::new();
    Processor::new("post.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve(files, (), &Ambient::default())
        .validate(&[])
        .load(context(), (), ())
        .expand(options)
}

const HELLO: &str = "```js component=hello\nexport default (target) => { target.textContent = \"the post's\"; };\n```\n\n:: hello[Hi]\n";

#[test]
fn an_examples_components_are_its_own_numbered_by_it() {
    let example = "````compmd {render=output}\n```js component=hello\nexport default (target) => { target.textContent = \"the example's\"; };\n```\n\n:: hello[Hi]\n````\n";
    let text = format!("{HELLO}\n{example}");
    let processor = process(&text, &Options::default());
    let all = processor.host_components();
    let components: Vec<(&str, Option<usize>, bool)> = all
        .iter()
        .map(|c| (c.name.as_str(), c.example, c.code.contains("the example's")))
        .collect();
    assert_eq!(
        components,
        [("hello", None, false), ("hello", Some(1), true)]
    );
    assert!(all.iter().all(|c| c.language == Language::Js));
    let json = serde_json::to_value(processor.expanded()).expect("a tree");
    let example = json["children"]
        .as_array()
        .and_then(|children| children.iter().find(|node| node["type"] == "code"))
        .expect("the example");
    assert_eq!(example["document"]["example"], 1);
    assert_eq!(example["document"]["children"][0]["name"], "hello");
}

#[test]
fn origins_and_diagnostics_are_at_their_places_in_the_document() {
    let text = "Intro.\n\n> ```compmd {render=output}\n> # Title\n> {{md.doc.missing}}\n> ```\n";
    let processor = process(
        text,
        &Options {
            origins: true,
            ..Options::default()
        },
    );
    let found: Vec<(&str, usize, usize)> = processor
        .diagnostics()
        .iter()
        .map(|d| (d.code.as_str(), d.range.start.line, d.range.start.column))
        .collect();
    assert_eq!(
        found,
        [("F014", 5, 5)],
        "in the quote, past `> ` and the braces"
    );
    let json = serde_json::to_value(processor.expanded()).expect("a tree");
    let heading = &json["children"][1]["children"][0]["document"]["children"][0];
    assert_eq!(heading["type"], "heading");
    assert_eq!(heading["origin"]["start"]["line"], 4);
    assert_eq!(heading["origin"]["start"]["column"], 3);
    let offset = text.find("# Title").expect("the heading");
    assert_eq!(heading["origin"]["start"]["offset"], offset);
}

#[test]
fn a_reload_keeps_the_examples_and_their_diagnostics_once() {
    let text = "```compmd {render=output}\n{{md.doc.missing}}\n```\n";
    let processor = process(text, &Options::default());
    assert_eq!(processor.diagnostics().len(), 1);
    let reloaded = processor.reload(context(), (), ());
    assert_eq!(reloaded.diagnostics().len(), 1);
    let json = serde_json::to_value(reloaded.expanded()).expect("a tree");
    assert_eq!(json["children"][0]["document"]["example"], 1);
}

#[test]
fn what_an_example_imports_is_needed_too() {
    let text = "```compmd {render=output}\n---\nmd:\n  import:\n    - from: \"@kit/charts@1\"\n      names: [chart]\n---\n:: chart[Sales]\n```\n";
    let validated = Processor::new("post.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve(HashMap::<String, String>::new(), (), &Ambient::default())
        .validate(&[]);
    let needed = validated.sources(&context());
    assert_eq!(needed.packs.len(), 1);
    assert_eq!(needed.packs[0].pack.to_string(), "@kit/charts");
}
