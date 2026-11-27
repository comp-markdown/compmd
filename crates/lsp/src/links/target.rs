//! A link's destination as the server follows it ([the language server], *Links and headings*): a project file, a
//! heading of it, or both; a URL with a scheme (`https:`, `mailto:`) is never followed.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use std::path::{Component, Path, PathBuf};

use percent_encoding::percent_decode_str;

/// Where a link leads: a file of the project, a heading's anchor, or both.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Target {
    /// The file, resolved; `None` for the document's own (`#anchor`).
    pub(crate) file: Option<PathBuf>,
    /// The anchor after `#`, percent-decoded; `None` without one. A passage (`#:~:text=…`) is no anchor.
    pub(crate) anchor: Option<String>,
}

/// Where the destination `url` of a link in the document at `document` leads, under the project root `root`; `None`
/// for a URL with a scheme, a passage alone, or an empty destination.
pub(crate) fn target_of(url: &str, document: &Path, root: &Path) -> Option<Target> {
    if url.is_empty() || has_scheme(url) {
        return None;
    }
    let (path, fragment) = url
        .split_once('#')
        .map_or((url, None), |(path, fragment)| (path, Some(fragment)));
    let path = path.split_once('?').map_or(path, |(path, _)| path);
    let anchor = fragment
        .filter(|fragment| !fragment.is_empty() && !fragment.starts_with(":~:"))
        .map(|fragment| {
            percent_decode_str(fragment)
                .decode_utf8_lossy()
                .into_owned()
        });
    let file = (!path.is_empty()).then(|| {
        resolved(
            &percent_decode_str(path).decode_utf8_lossy(),
            document,
            root,
        )
    });
    (file.is_some() || anchor.is_some()).then_some(Target { file, anchor })
}

/// Whether `url` starts with a scheme, `name:` (RFC 3986: a letter, then letters, digits, `+`, `-` and `.`).
fn has_scheme(url: &str) -> bool {
    let Some((scheme, _)) = url.split_once(':') else {
        return false;
    };
    let mut characters = scheme.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        })
}

/// `path`, relative to the document's folder, or to the root when it starts with `/`, with `.` and `..` resolved.
fn resolved(path: &str, document: &Path, root: &Path) -> PathBuf {
    let base = match path.strip_prefix('/') {
        Some(_) => root.to_path_buf(),
        None => document
            .parent()
            .map_or_else(|| root.to_path_buf(), Path::to_path_buf),
    };
    let mut resolved = PathBuf::new();
    for component in base.join(path.trim_start_matches('/')).components() {
        match component {
            Component::ParentDir => {
                resolved.pop();
            }
            Component::CurDir => {}
            other => resolved.push(other),
        }
    }
    resolved
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(url: &str) -> Option<Target> {
        target_of(url, Path::new("/site/posts/harbour.md"), Path::new("/site"))
    }

    #[test]
    fn a_relative_path_is_a_file_of_the_project_and_a_fragment_its_anchor() {
        assert_eq!(
            target("../tides.comp.md#high-water"),
            Some(Target {
                file: Some(PathBuf::from("/site/tides.comp.md")),
                anchor: Some("high-water".into()),
            })
        );
        assert_eq!(
            target("./notes%20on%20knots.md").and_then(|target| target.file),
            Some(PathBuf::from("/site/posts/notes on knots.md"))
        );
        assert_eq!(
            target("/about.md?draft=1").and_then(|target| target.file),
            Some(PathBuf::from("/site/about.md")),
            "from the root, without its query"
        );
    }

    #[test]
    fn an_anchor_alone_is_the_documents_own_heading() {
        assert_eq!(
            target("#caf%C3%A9--bar"),
            Some(Target {
                file: None,
                anchor: Some("café--bar".into()),
            })
        );
    }

    #[test]
    fn urls_passages_and_nothing_arent_followed() {
        for url in [
            "https://example.com/a.md",
            "mailto:a@b.c",
            "",
            "#",
            "#:~:text=tide",
        ] {
            assert_eq!(target(url), None, "{url}");
        }
        assert_eq!(
            target("tides.md#:~:text=high").and_then(|target| target.anchor),
            None,
            "a passage is no anchor"
        );
        assert!(!has_scheme("1http:x"), "a scheme starts with a letter");
        assert!(!has_scheme("a b:c"));
    }
}
