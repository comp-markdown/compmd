//! Component analysis (spec §7.1, §13.1): W104 for each host-language component of a document whose signature fence
//! disagrees with what the host's reader finds in its code.

use super::{Loader, location};
use crate::analysis::mismatches;
use crate::diagnostic::{Code, Location};
use crate::resolve::{Body, DocumentId, Language, Origin};
use crate::tree::Position;

/// The languages whose components take props: a binding passes params to them (spec §7.6).
const ANALYSED: [Language; 3] = [Language::Jsx, Language::Tsx, Language::Svelte];

/// A file's name as W104 writes it: its path in the root folder or in its pack.
fn file_name(origin: &Origin) -> String {
    match origin {
        Origin::File(path) => path.as_str().to_owned(),
        Origin::Pack(place) => place.path.as_str().to_owned(),
        Origin::Unplaced(path) => path.clone(),
    }
}

/// Where `param` is declared inside the signature fence at `fence`, in `text`: its line, at its name; the fence when
/// it isn't found there, or for a mismatch about no param.
fn place_of(text: &str, fence: Option<Position>, param: Option<&str>) -> Location {
    let (Some(fence), Some(param)) = (fence, param) else {
        return location(fence);
    };
    let declared = |line: &str| {
        let name = line.trim_start();
        let indent = line.chars().count() - name.chars().count();
        let rest = name.strip_prefix(param)?;
        (indent > 0 && rest.trim_start().starts_with(':')).then_some(indent)
    };
    text.split('\n')
        .enumerate()
        .map(|(index, line)| (index + 1, line))
        .filter(|(number, _)| *number > fence.start.line && *number < fence.end.line)
        .find_map(|(number, line)| declared(line).map(|indent| Location::new(number, indent + 1)))
        .unwrap_or_else(|| location(Some(fence)))
}

impl Loader<'_, '_> {
    /// W104 for each host-language component the document `id` gives a signature fence, where the host's reader
    /// finds that its code disagrees: one defined in a fence, or a component file the document imports.
    pub(super) fn analyse(&mut self, id: DocumentId) {
        let document = self.document(id);
        let mut found = Vec::new();
        for component in self
            .components
            .iter()
            .filter(|component| component.document == id)
        {
            let Some(signature) = &component.signature else {
                continue;
            };
            let (language, code, file) = match &component.body {
                Body::Host {
                    language, source, ..
                } => (
                    Language::from_name(language),
                    Some(source.clone()),
                    file_name(&document.origin),
                ),
                Body::File { origin, language } => {
                    (Some(*language), self.store.read(origin), file_name(origin))
                }
                Body::Compmd(_) | Body::Processor => continue,
            };
            // Empty code is a component the host gives the renderer otherwise (compiled ahead, a built module): there
            // is nothing to read.
            let (Some(language), Some(code)) = (
                language.filter(|language| ANALYSED.contains(language)),
                code.filter(|code| !code.trim().is_empty()),
            ) else {
                continue;
            };
            let Some(props) = self.store.props.props(language, &code) else {
                continue;
            };
            for mismatch in mismatches(&file, signature, &props) {
                let at = place_of(
                    &document.text,
                    component.signature_at,
                    mismatch.param.as_deref(),
                );
                found.push((at, mismatch.message));
            }
        }
        for (at, message) in found {
            self.report(id, Code::W104, at, message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::Point;

    /// The fence of lines 2 to 6.
    const FENCE: Option<Position> = Some(Position {
        start: Point {
            line: 2,
            column: 1,
            offset: 0,
        },
        end: Point {
            line: 6,
            column: 1,
            offset: 0,
        },
    });

    const TEXT: &str = "# A\n```compmd component=c signature\nparams:\n  label: string\n  labelled : number\n```\nlabel: x\n";

    #[test]
    fn a_param_is_placed_at_its_name_in_the_fence() {
        assert_eq!(place_of(TEXT, FENCE, Some("label")), Location::new(4, 3));
        assert_eq!(place_of(TEXT, FENCE, Some("labelled")), Location::new(5, 3));
    }

    #[test]
    fn anything_else_is_placed_at_the_fence() {
        assert_eq!(place_of(TEXT, FENCE, Some("size")), Location::new(2, 1));
        assert_eq!(place_of(TEXT, FENCE, None), Location::new(2, 1));
        assert_eq!(
            place_of(TEXT, FENCE, Some("params")),
            Location::new(2, 1),
            "not indented"
        );
        assert_eq!(place_of(TEXT, None, Some("label")), Location::new(1, 1));
    }

    #[test]
    fn a_file_is_named_by_its_path() {
        assert_eq!(
            file_name(&Origin::Unplaced("./a.comp.md".into())),
            "./a.comp.md"
        );
    }
}
