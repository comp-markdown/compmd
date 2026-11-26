//! What the features' tests share.

use std::rc::Rc;

use compmd::context::RenderContext;
use compmd::processor::Expanded;
use compmd::{Ambient, ParseOptions, Processor};
use lsp_types::Uri;

use super::Asked;
use crate::encoding::Encoding;
use crate::pack_components::PackComponent;
use crate::places::Places;

/// `text`, processed as the server processes a document without a project.
pub(crate) fn processed(text: &str) -> Processor<Expanded> {
    let context = RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    };
    Processor::new("a.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve((), (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&compmd::expand::Options::default())
}

/// What `answer` takes about `text`, processed with no project, and with the packs' `components`, given to `answer`.
pub(crate) fn asking<R>(
    text: &str,
    processor: &Processor<Expanded>,
    components: &[PackComponent],
    answer: impl FnOnce(&Asked) -> R,
) -> R {
    let uri: Uri = "file:///site/a.comp.md".parse().expect("a URI");
    let places = Places::default();
    answer(&Asked {
        processor: Some(processor),
        uri: &uri,
        text,
        encoding: Encoding::Utf16,
        places: &places,
        pack_components: components,
        unregistered: None,
    })
}

/// The code point offset of the first `|` in `marked`, and the text without it.
pub(crate) fn caret(marked: &str) -> (String, usize) {
    let at = marked.chars().position(|c| c == '|').expect("a caret");
    (marked.replacen('|', "", 1), at)
}
