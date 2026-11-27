//! A project file as a link from a document writes it ([the language server], *Links and headings*, completion):
//! relative to the document's folder, with `/` between its parts and what a destination can't hold percent-encoded.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use std::path::{Component, Path};

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

/// What a link's destination writes percent-encoded: spaces and what ends or splits it.
const IN_A_DESTINATION: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'(')
    .add(b')')
    .add(b'<')
    .add(b'>')
    .add(b'#')
    .add(b'%');

/// `file` as a link from a document in `folder` writes it: `notes.md`, `../tides/high.md`.
pub(crate) fn relative_path(file: &Path, folder: &Path) -> String {
    let file: Vec<Component> = file.components().collect();
    let folder: Vec<Component> = folder.components().collect();
    let shared = file.iter().zip(&folder).take_while(|(a, b)| a == b).count();
    let up = std::iter::repeat_n("..".to_owned(), folder.len() - shared);
    let down = file[shared..].iter().map(|part| {
        utf8_percent_encode(&part.as_os_str().to_string_lossy(), IN_A_DESTINATION).to_string()
    });
    up.chain(down).collect::<Vec<_>>().join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_is_written_from_the_documents_folder_up_and_down() {
        let folder = Path::new("/site/posts");
        assert_eq!(
            relative_path(Path::new("/site/posts/tides.md"), folder),
            "tides.md"
        );
        assert_eq!(
            relative_path(Path::new("/site/notes/knots.comp.md"), folder),
            "../notes/knots.comp.md"
        );
        assert_eq!(
            relative_path(Path::new("/site/posts/2026/high water.md"), folder),
            "2026/high%20water.md"
        );
    }
}
