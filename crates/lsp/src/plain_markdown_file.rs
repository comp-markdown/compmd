//! Which files the server opens as plain Markdown ([a document drawn in the editor], decision 13): a `.md` file that
//! isn't a `.comp.md` one. The processor then reads it as the spec says (section 2): CompMD only with an `md` key.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use std::path::Path;

/// Whether the file at `path` is opened as plain Markdown.
pub(crate) fn is_plain_markdown_file(path: &Path) -> bool {
    let markdown = path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"));
    let stem = path.file_stem().map(Path::new);
    let comp = stem
        .and_then(Path::extension)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("comp"));
    markdown && !comp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_md_file_is_plain_markdown_and_a_comp_md_one_is_not() {
        assert!(is_plain_markdown_file(Path::new("/site/notes.md")));
        assert!(is_plain_markdown_file(Path::new("README.MD")));
        assert!(!is_plain_markdown_file(Path::new("/site/notes.comp.md")));
        assert!(!is_plain_markdown_file(Path::new("/site/Notes.Comp.md")));
        assert!(!is_plain_markdown_file(Path::new("/site/notes.txt")));
        assert!(!is_plain_markdown_file(Path::new("/")));
    }
}
