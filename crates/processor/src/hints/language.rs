//! The hint on a fenced code block's language (spec §7.7): the code renderer it selects, if any, and the languages
//! the document's code renderers render. Every fence's language has one, an empty one too, so an editor can offer
//! the languages before one is written.

use super::{Completion, Found, Place, Subject};
use crate::resolve::Renderer;

/// The hint on `language`, at code points `start..end` of the place's line.
pub(super) fn hint(place: &Place, language: &str, start: usize, end: usize) -> Found {
    let renderer = match place.document.renderers.get(language) {
        Some(Renderer::Component(name, _)) => Some(name.as_str()),
        _ => None,
    };
    Found {
        start,
        end,
        subject: Subject::Language {
            language: language.to_string(),
            renderer: renderer.map(ToString::to_string),
            component: renderer.and_then(|name| place.component_hint(name)),
        },
        completions: rendered(place),
    }
}

/// The languages the document's code renderers render, each with the renderer's name.
fn rendered(place: &Place) -> Vec<Completion> {
    place
        .document
        .renderers
        .iter()
        .filter_map(|(language, renderer)| {
            let Renderer::Component(name, _) = renderer else {
                return None;
            };
            Some(Completion::new(
                language,
                Some(format!("drawn by `{name}`")),
            ))
        })
        .collect()
}
