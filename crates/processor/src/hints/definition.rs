//! Where what a place names is defined, for an editor's "go to definition": a component's name, or a code block's
//! language, to its definition fence, or to the file or pack document that exports it; an import's `from` to what it loads; `md.data.<name>` to
//! its data import.

use super::{Place, Subject, found};
use crate::diagnostic::{Location, Range};
use crate::resolve::Resolved;
use serde::Serialize;

/// Where something is defined.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Defined {
    /// The file, as diagnostics name it (a path in the root, a pack's place); `None` for the processed document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Where in it; `None` for the file as a whole.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
}

/// Where what code point `offset` of the processed document of `resolved` names is defined.
pub(crate) fn at(resolved: &Resolved, offset: usize) -> Option<Defined> {
    let place = Place::at(resolved, offset);
    if let Some(imported) = import(&place) {
        return Some(imported);
    }
    match found(&place)?.subject {
        Subject::Component {
            component: Some(component),
            ..
        }
        | Subject::Language {
            component: Some(component),
            ..
        } => Some(Defined {
            file: component.file,
            range: component.defined,
        }),
        Subject::Path { path, .. } => {
            let name = path
                .strip_prefix("md.data.")
                .filter(|name| !name.contains('.'))?;
            let data = place
                .document
                .settings
                .data
                .iter()
                .find(|data| data.name == name)?;
            Some(Defined {
                file: None,
                range: Some(point(data.from.at)),
            })
        }
        _ => None,
    }
}

/// What the import whose `from` is on the place's line loaded.
fn import(place: &Place) -> Option<Defined> {
    let (_, origin) = place
        .document
        .imported
        .iter()
        .find(|(at, _)| at.line == place.line)?;
    let main = &place.resolved.document(place.resolved.main).origin;
    Some(Defined {
        file: (origin != main).then(|| origin.to_string()),
        range: None,
    })
}

/// The empty range at `at`.
fn point(at: Location) -> Range {
    Range {
        start: at,
        end: Some(at),
    }
}
