//! Where a wiki link leads ([the language server], *Links and headings*, as Marksman and Obsidian resolve them):
//! `[[Page#Heading|label]]` names a project file by its name without `.comp.md` or `.md`, any case, the nearest to the
//! document first when two share one, and a heading of it by its text, any case, after `#`. A page written with
//! folders (`notes/Page`) names the file whose path ends with them; `[[#Heading]]` is a heading of the document.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use std::path::{Component, Path, PathBuf};

use crate::links::headings::Heading;

/// A wiki link's target, `Page#Heading`, in its parts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WikiTarget {
    /// The page, as written; empty for the document's own heading.
    pub(crate) page: String,
    /// The heading after `#`, when there is one.
    pub(crate) heading: Option<String>,
}

/// The parts of a wiki link's `target`.
pub(crate) fn wiki_target(target: &str) -> WikiTarget {
    let (page, heading) = target
        .split_once('#')
        .map_or((target, None), |(page, heading)| (page, Some(heading)));
    WikiTarget {
        page: page.trim().to_owned(),
        heading: heading
            .map(str::trim)
            .filter(|heading| !heading.is_empty())
            .map(str::to_owned),
    }
}

/// The name a wiki link writes for `file`: its file name without `.comp.md` or `.md`.
pub(crate) fn page_name(file: &Path) -> String {
    let name = file
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
    without_extension(&name).to_owned()
}

/// The file among `files` that `page`, written in a wiki link of the document at `document` under `root`, names; the
/// document itself for an empty page.
pub(crate) fn wiki_file<'a>(
    page: &str,
    document: &Path,
    root: &Path,
    files: impl IntoIterator<Item = &'a Path>,
) -> Option<PathBuf> {
    if page.is_empty() {
        return Some(document.to_path_buf());
    }
    let wanted = without_extension(page.trim_start_matches('/')).to_lowercase();
    let folder = document.parent().unwrap_or(root);
    files
        .into_iter()
        .filter(|file| {
            let path = page_path(file, root).to_lowercase();
            path == wanted || path.ends_with(&format!("/{wanted}"))
        })
        .min_by_key(|file| distance(file, folder))
        .map(Path::to_path_buf)
}

/// Whether `written`, a wiki link's heading, names `heading`: its text, any case, or its anchor.
pub(crate) fn names_heading(written: &str, heading: &Heading) -> bool {
    written.to_lowercase() == heading.text.to_lowercase() || written == heading.anchor
}

/// `file`'s path from `root`, `/`-separated, without `.comp.md` or `.md`: what a wiki link writes for it with its
/// folders.
pub(crate) fn page_path(file: &Path, root: &Path) -> String {
    let inside = file.strip_prefix(root).unwrap_or(file);
    let parts: Vec<String> = inside
        .components()
        .filter_map(|part| match part {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    without_extension(&parts.join("/")).to_owned()
}

/// `name` without `.comp.md` or `.md`, any case.
fn without_extension(name: &str) -> &str {
    [".comp.md", ".md"]
        .iter()
        .find_map(|extension| {
            let cut = name.len().checked_sub(extension.len())?;
            (name.is_char_boundary(cut) && name[cut..].eq_ignore_ascii_case(extension))
                .then(|| &name[..cut])
        })
        .unwrap_or(name)
}

/// How many folders up and down `file` is from `folder`.
fn distance(file: &Path, folder: &Path) -> usize {
    let file: Vec<Component> = file.components().collect();
    let folder: Vec<Component> = folder.components().collect();
    let shared = file.iter().zip(&folder).take_while(|(a, b)| a == b).count();
    file.len() - shared + folder.len() - shared
}

#[cfg(test)]
mod tests {
    use compmd::tree::{Point, Position};

    use super::*;

    const ROOT: &str = "/site";

    fn files() -> Vec<PathBuf> {
        [
            "/site/Tides.md",
            "/site/posts/tides.comp.md",
            "/site/posts/harbour.md",
            "/site/notes/Knots and Bends.md",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect()
    }

    fn file(page: &str, document: &str) -> Option<PathBuf> {
        let files = files();
        wiki_file(
            page,
            Path::new(document),
            Path::new(ROOT),
            files.iter().map(PathBuf::as_path),
        )
    }

    #[test]
    fn a_target_is_its_page_and_heading() {
        assert_eq!(
            wiki_target("Tides # High water "),
            WikiTarget {
                page: "Tides".into(),
                heading: Some("High water".into())
            }
        );
        assert_eq!(wiki_target("#Top").page, "");
        assert_eq!(wiki_target("Tides#").heading, None);
    }

    #[test]
    fn a_page_is_a_files_name_any_case_the_nearest_first() {
        assert_eq!(
            file("tides", "/site/posts/harbour.md"),
            Some(PathBuf::from("/site/posts/tides.comp.md"))
        );
        assert_eq!(
            file("TIDES", "/site/index.md"),
            Some(PathBuf::from("/site/Tides.md"))
        );
        assert_eq!(
            file("knots and bends", "/site/index.md"),
            Some(PathBuf::from("/site/notes/Knots and Bends.md"))
        );
        assert_eq!(
            file("notes/Knots and Bends.md", "/site/index.md"),
            Some(PathBuf::from("/site/notes/Knots and Bends.md")),
            "with its folder and its extension"
        );
        assert_eq!(file("ots and Bends", "/site/index.md"), None);
        assert_eq!(
            file("", "/site/index.md"),
            Some(PathBuf::from("/site/index.md"))
        );
    }

    #[test]
    fn a_files_page_name_and_a_heading_by_its_text_or_anchor() {
        assert_eq!(page_name(Path::new("/site/posts/Tides.Comp.MD")), "Tides");
        assert_eq!(page_name(Path::new("/site/notes.txt")), "notes.txt");
        let point = Point {
            line: 1,
            column: 1,
            offset: 0,
        };
        let heading = Heading {
            anchor: "high-water".into(),
            depth: 2,
            text: "High Water".into(),
            position: Position {
                start: point,
                end: point,
            },
        };
        assert!(names_heading("high water", &heading));
        assert!(names_heading("high-water", &heading));
        assert!(!names_heading("Low water", &heading));
    }
}
