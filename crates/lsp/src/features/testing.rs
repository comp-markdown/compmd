//! What the features' tests share.

use std::path::Path;
use std::rc::Rc;

use compmd::context::RenderContext;
use compmd::processor::Expanded;
use compmd::{Ambient, ParseOptions, Processor};
use lsp_types::Uri;

use super::Asked;
use crate::encoding::Encoding;
use crate::pack_components::PackComponent;
use crate::places::Places;
use crate::plain_markdown_file::is_plain_markdown_file;
use crate::project_index::{IndexedFile, ProjectIndex};

/// `text`, processed as the server processes a document without a project.
pub(crate) fn processed(text: &str) -> Processor<Expanded> {
    processed_file("a.comp.md", text)
}

/// `text`, the file `name`, processed as the server processes a document without a project: a `.md` file as plain
/// Markdown unless it has an `md` key.
pub(crate) fn processed_file(name: &str, text: &str) -> Processor<Expanded> {
    let context = RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    };
    Processor::new(name, text)
        .parse(&ParseOptions {
            plain_markdown: is_plain_markdown_file(Path::new(name)),
            ..ParseOptions::default()
        })
        .resolve((), (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&compmd::expand::Options {
            origins: true,
            ..compmd::expand::Options::default()
        })
}

/// What `answer` takes about `text`, processed with no project, and with the packs' `components`, given to `answer`.
pub(crate) fn asking<R>(
    text: &str,
    processor: &Processor<Expanded>,
    components: &[PackComponent],
    answer: impl FnOnce(&Asked) -> R,
) -> R {
    asking_file("a.comp.md", text, processor, components, answer)
}

/// As [`asking`], for the file `name` of the site.
pub(crate) fn asking_file<R>(
    name: &str,
    text: &str,
    processor: &Processor<Expanded>,
    components: &[PackComponent],
    answer: impl FnOnce(&Asked) -> R,
) -> R {
    let uri: Uri = format!("file:///site/{name}").parse().expect("a URI");
    let path = Path::new("/site").join(name);
    let places = Places::default();
    answer(&Asked {
        processor: Some(processor),
        uri: &uri,
        plain_markdown_file: is_plain_markdown_file(Path::new(name)),
        path: &path,
        index: None,
        links: true,
        text,
        version: 1,
        encoding: Encoding::Utf16,
        places: &places,
        pack_components: components,
        unregistered: None,
    })
}

/// What `answer` takes about the file `name` of the project at `root`, which holds `files` (a path in it and its
/// text, written to the disk and indexed), the document's text being `text`.
pub(crate) fn asking_in_project<R>(
    root: &Path,
    files: &[(&str, &str)],
    (name, text): (&str, &str),
    answer: impl FnOnce(&Asked) -> R,
) -> R {
    let _ = std::fs::remove_dir_all(root);
    let index = ProjectIndex::new(root.to_path_buf());
    for (file, content) in files.iter().chain([&(name, text)]) {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().expect("a folder")).expect("its folder");
        std::fs::write(&path, content).expect("written");
        index.set(path.clone(), IndexedFile::read(&path, content));
    }
    let path = root.join(name);
    let processor = processed_file(name, text);
    let uri = crate::features::link_targets::uri_of(&path).expect("a URI");
    let places = Places::default();
    answer(&Asked {
        processor: Some(&processor),
        uri: &uri,
        plain_markdown_file: is_plain_markdown_file(&path),
        path: &path,
        index: Some(&index),
        links: true,
        text,
        version: 1,
        encoding: Encoding::Utf16,
        places: &places,
        pack_components: &[],
        unregistered: None,
    })
}

/// A folder of its own for the test `name`.
pub(crate) fn project_folder(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("compmd-lsp-links-{}-{name}", std::process::id()))
}

/// The code point offset of the first `|` in `marked`, and the text without it.
pub(crate) fn caret(marked: &str) -> (String, usize) {
    let at = marked.chars().position(|c| c == '|').expect("a caret");
    (marked.replacen('|', "", 1), at)
}
