//! Where what a caret names is defined: a component, an import, a data import.

use super::CARET;
use crate::context::RenderContext;
use crate::expand::Options;
use crate::hints::Defined;
use crate::{Ambient, ParseOptions, Processor};
use std::collections::HashMap;
use std::rc::Rc;

/// Where what the caret of `text` names is defined, with `files` beside it: its file and the line it starts on.
fn defined(text: &str, files: &[(&str, &str)]) -> Option<(Option<String>, Option<usize>)> {
    let offset = text.chars().position(|c| c == CARET).expect("a caret");
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
    let processed = Processor::new("a.comp.md", text.replacen(CARET, "", 1))
        .parse(&ParseOptions::default())
        .resolve(files, (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&Options::default());
    let Defined { file, range } = processed.definition_at(offset)?;
    Some((file, range.map(|range| range.start.line)))
}

/// A file that exports `note`, defined on its line 6.
const NOTES: &str =
    "---\nmd:\n  export:\n    names: \"*\"\n---\n```compmd component=note\n---\nA note.\n```\n";

#[test]
fn a_components_name_goes_to_its_definition_here_or_in_the_file_that_exports_it() {
    let here = "```compmd component=note\n---\nA note.\n```\n\n:: no‸te[N]\n";
    assert_eq!(defined(here, &[]), Some((None, Some(1))));
    let imported = "---\nmd:\n  import:\n    - from: ./notes.comp.md\n      names: \"*\"\n---\n\n:: no‸te[N]\n";
    assert_eq!(
        defined(imported, &[("notes.comp.md", NOTES)]),
        Some((Some("notes.comp.md".into()), Some(6)))
    );
    assert_eq!(
        defined(":: no‸pe[N]\n", &[]),
        None,
        "a name that stands for nothing"
    );
}

#[test]
fn an_imports_from_goes_to_what_it_loads() {
    let text = "---\nmd:\n  import:\n    - from: ./no‸tes.comp.md\n      names: \"*\"\n    - from: ./chart.jsx\n      names: \"*\"\n---\n";
    assert_eq!(
        defined(
            text,
            &[
                ("notes.comp.md", NOTES),
                ("chart.jsx", "export default () => null;")
            ]
        ),
        Some((Some("notes.comp.md".into()), None))
    );
    let file = text
        .replacen(CARET, "", 1)
        .replacen("./chart", "./ch‸art", 1);
    assert_eq!(
        defined(
            &file,
            &[
                ("notes.comp.md", NOTES),
                ("chart.jsx", "export default () => null;")
            ]
        ),
        Some((Some("chart.jsx".into()), None))
    );
    let missing = "---\nmd:\n  import:\n    - from: ./no‸ne.comp.md\n      names: \"*\"\n---\n";
    assert_eq!(defined(missing, &[]), None, "nothing loaded");
}

#[test]
fn a_data_path_goes_to_its_data_import() {
    let text = "---\nmd:\n  data:\n    - from: ./sales.csv\n      as: sales\n---\n\n{{ md.data.sa‸les }}\n";
    assert_eq!(
        defined(text, &[("sales.csv", "a\n1\n")]),
        Some((None, Some(4)))
    );
    let field =
        "---\nmd:\n  data:\n    - from: ./sales.csv\n      as: sales\n---\n\n{{ md.doc.ti‸tle }}\n";
    assert_eq!(defined(field, &[("sales.csv", "a\n1\n")]), None, "not data");
}
